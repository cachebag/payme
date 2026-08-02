use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            savings REAL NOT NULL DEFAULT 0,
            savings_goal REAL NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("ALTER TABLE users ADD COLUMN savings REAL NOT NULL DEFAULT 0")
        .execute(pool)
        .await
        .ok();

    sqlx::query("ALTER TABLE users ADD COLUMN retirement_savings REAL NOT NULL DEFAULT 0")
        .execute(pool)
        .await
        .ok();

    sqlx::query("ALTER TABLE users ADD COLUMN savings_goal REAL NOT NULL DEFAULT 0")
        .execute(pool)
        .await
        .ok();

    sqlx::query("UPDATE users SET retirement_savings = roth_ira WHERE retirement_savings = 0 AND roth_ira IS NOT NULL AND roth_ira > 0")
        .execute(pool)
        .await
        .ok();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS fixed_expenses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            label TEXT NOT NULL,
            amount REAL NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("ALTER TABLE fixed_expenses ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0")
        .execute(pool)
        .await
        .ok();

    sqlx::query("UPDATE fixed_expenses SET sort_order = id WHERE sort_order = 0 AND NOT EXISTS (SELECT 1 FROM fixed_expenses WHERE sort_order <> 0)")
        .execute(pool)
        .await
        .ok();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS budget_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            label TEXT NOT NULL,
            default_amount REAL NOT NULL,
            color TEXT NOT NULL DEFAULT '#71717a',
            sort_order INTEGER NOT NULL DEFAULT 0,
            archived_at TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    let _ = sqlx::query(
        "ALTER TABLE budget_categories ADD COLUMN color TEXT NOT NULL DEFAULT '#71717a'",
    )
    .execute(pool)
    .await;

    sqlx::query("ALTER TABLE budget_categories ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0")
        .execute(pool)
        .await
        .ok();

    sqlx::query("UPDATE budget_categories SET sort_order = id WHERE sort_order = 0 AND NOT EXISTS (SELECT 1 FROM budget_categories WHERE sort_order <> 0)")
        .execute(pool)
        .await
        .ok();

    // Retiring a category sets this instead of deleting the row: `monthly_budgets` and `items`
    // reference categories with ON DELETE CASCADE, so dropping the row would erase every
    // month's allocations and transactions for it, history included.
    sqlx::query("ALTER TABLE budget_categories ADD COLUMN archived_at TEXT")
        .execute(pool)
        .await
        .ok();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS months (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            year INTEGER NOT NULL,
            month INTEGER NOT NULL,
            is_closed INTEGER NOT NULL DEFAULT 0,
            closed_at TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE(user_id, year, month)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS income_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL,
            label TEXT NOT NULL,
            amount REAL NOT NULL,
            paid_on TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query("ALTER TABLE income_entries ADD COLUMN paid_on TEXT")
        .execute(pool)
        .await
        .ok();

    sqlx::query("ALTER TABLE income_entries ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0")
        .execute(pool)
        .await
        .ok();

    sqlx::query("UPDATE income_entries SET sort_order = id WHERE sort_order = 0 AND NOT EXISTS (SELECT 1 FROM income_entries WHERE sort_order <> 0)")
        .execute(pool)
        .await
        .ok();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS monthly_budgets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL,
            category_id INTEGER NOT NULL,
            allocated_amount REAL NOT NULL,
            FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES budget_categories(id) ON DELETE CASCADE,
            UNIQUE(month_id, category_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL,
            category_id INTEGER,
            description TEXT NOT NULL,
            amount REAL NOT NULL,
            spent_on TEXT NOT NULL,
            savings_destination TEXT NOT NULL DEFAULT 'none',
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE,
            FOREIGN KEY (category_id) REFERENCES budget_categories(id) ON DELETE SET NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    let _ = sqlx::query(
        "ALTER TABLE items ADD COLUMN savings_destination TEXT NOT NULL DEFAULT 'none'",
    )
    .execute(pool)
    .await;

    sqlx::query("ALTER TABLE items ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0")
        .execute(pool)
        .await
        .ok();

    sqlx::query("UPDATE items SET savings_destination = 'none' WHERE savings_destination = '' OR savings_destination IS NULL")
        .execute(pool)
        .await?;

    sqlx::query("UPDATE items SET sort_order = id WHERE sort_order = 0 AND NOT EXISTS (SELECT 1 FROM items WHERE sort_order <> 0)")
        .execute(pool)
        .await
        .ok();

    // items.category_id used to be NOT NULL. A transaction whose category is removed from
    // its month is now simply uncategorized (NULL) instead of clinging to a ghost category.
    // SQLite cannot drop NOT NULL in place, so legacy tables are rebuilt once.
    let items_sql: Option<String> =
        sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'items'")
            .fetch_optional(pool)
            .await?;

    if items_sql.is_some_and(|sql| sql.contains("category_id INTEGER NOT NULL")) {
        // The whole rebuild runs inside one transaction on one connection: every statement
        // either lands together or rolls back together, so a failure can never leave the
        // database between tables. (Statements straight off the pool may hit different
        // connections, which is how a partial rebuild once stranded the data mid-rename.)
        let mut tx = pool.begin().await?;

        // A crashed earlier attempt may have left the scratch table behind.
        sqlx::query("DROP TABLE IF EXISTS items_rebuild")
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE items_rebuild (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                month_id INTEGER NOT NULL,
                category_id INTEGER,
                description TEXT NOT NULL,
                amount REAL NOT NULL,
                spent_on TEXT NOT NULL,
                savings_destination TEXT NOT NULL DEFAULT 'none',
                sort_order INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE,
                FOREIGN KEY (category_id) REFERENCES budget_categories(id) ON DELETE SET NULL
            )
            "#,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO items_rebuild (id, month_id, category_id, description, amount, spent_on, savings_destination, sort_order)
             SELECT id, month_id, category_id, description, amount, spent_on, savings_destination, sort_order FROM items",
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query("DROP TABLE items").execute(&mut *tx).await?;
        sqlx::query("ALTER TABLE items_rebuild RENAME TO items")
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
    }

    // Repair transactions orphaned by the era when deleting a category removed its row
    // outright: they still point at ids that no longer exist. Make them uncategorized.
    sqlx::query(
        "UPDATE items SET category_id = NULL WHERE category_id IS NOT NULL AND category_id NOT IN (SELECT id FROM budget_categories)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS monthly_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL UNIQUE,
            pdf_data BLOB NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS monthly_fixed_expenses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL,
            label TEXT NOT NULL,
            amount REAL NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            group_id INTEGER,
            FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "ALTER TABLE monthly_fixed_expenses ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await
    .ok();

    // Links the "same" fixed expense across months so edits can propagate forward.
    // Internal only; never exposed through the API.
    sqlx::query("ALTER TABLE monthly_fixed_expenses ADD COLUMN group_id INTEGER")
        .execute(pool)
        .await
        .ok();

    sqlx::query("UPDATE monthly_fixed_expenses SET sort_order = id WHERE sort_order = 0 AND NOT EXISTS (SELECT 1 FROM monthly_fixed_expenses WHERE sort_order <> 0)")
        .execute(pool)
        .await
        .ok();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS monthly_savings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            month_id INTEGER NOT NULL UNIQUE,
            savings REAL NOT NULL DEFAULT 0,
            retirement_savings REAL NOT NULL DEFAULT 0,
            savings_goal REAL NOT NULL DEFAULT 0,
            FOREIGN KEY (month_id) REFERENCES months(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS custom_savings_goals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            current_amount REAL NOT NULL DEFAULT 0,
            target_amount REAL NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "ALTER TABLE custom_savings_goals ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await
    .ok();

    sqlx::query("UPDATE custom_savings_goals SET sort_order = id WHERE sort_order = 0 AND NOT EXISTS (SELECT 1 FROM custom_savings_goals WHERE sort_order <> 0)")
        .execute(pool)
        .await
        .ok();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS retirement_breakdown_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            label TEXT NOT NULL,
            amount REAL NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "ALTER TABLE retirement_breakdown_items ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
    )
    .execute(pool)
    .await
    .ok();

    sqlx::query("UPDATE retirement_breakdown_items SET sort_order = id WHERE sort_order = 0 AND NOT EXISTS (SELECT 1 FROM retirement_breakdown_items WHERE sort_order <> 0)")
        .execute(pool)
        .await
        .ok();

    let existing_months: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT id, user_id FROM months WHERE id NOT IN (SELECT DISTINCT month_id FROM monthly_fixed_expenses)",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for (month_id, user_id) in existing_months {
        let fixed_expenses: Vec<(String, f64, i64)> =
            sqlx::query_as("SELECT label, amount, sort_order FROM fixed_expenses WHERE user_id = ? ORDER BY sort_order, id")
                .bind(user_id)
                .fetch_all(pool)
                .await
                .unwrap_or_default();

        for (label, amount, sort_order) in fixed_expenses {
            sqlx::query(
                "INSERT INTO monthly_fixed_expenses (month_id, label, amount, sort_order) VALUES (?, ?, ?, ?)",
            )
            .bind(month_id)
            .bind(&label)
            .bind(amount)
            .bind(sort_order)
            .execute(pool)
            .await
            .ok();
        }

        let user_savings: Option<(f64, f64, f64)> = sqlx::query_as(
            "SELECT savings, retirement_savings, savings_goal FROM users WHERE id = ?",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        if let Some((savings, retirement_savings, savings_goal)) = user_savings {
            sqlx::query(
                "INSERT INTO monthly_savings (month_id, savings, retirement_savings, savings_goal) VALUES (?, ?, ?, ?)",
            )
            .bind(month_id)
            .bind(savings)
            .bind(retirement_savings)
            .bind(savings_goal)
            .execute(pool)
            .await
            .ok();
        }
    }

    // Backfill group_id for rows created before the column existed (including the
    // template-seeded rows inserted just above). Rows belonging to the same user with the
    // same label are assumed to be the same recurring expense and share a group.
    sqlx::query(
        r#"
        UPDATE monthly_fixed_expenses SET group_id = (
            SELECT MIN(other.id)
            FROM monthly_fixed_expenses other
            JOIN months om ON om.id = other.month_id
            JOIN months tm ON tm.id = monthly_fixed_expenses.month_id
            WHERE om.user_id = tm.user_id AND other.label = monthly_fixed_expenses.label
        )
        WHERE group_id IS NULL
        "#,
    )
    .execute(pool)
    .await
    .ok();

    // A month only seeds the categories that existed when it was created, so a category added
    // afterwards (or created without naming a month) leaves already-open months missing a
    // budget line for it. Give every open month a line for every live category; closed months
    // are settled history and keep whatever they were closed with.
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO monthly_budgets (month_id, category_id, allocated_amount)
        SELECT m.id, bc.id, bc.default_amount
        FROM months m
        JOIN budget_categories bc ON bc.user_id = m.user_id AND bc.archived_at IS NULL
        WHERE m.is_closed = 0
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
