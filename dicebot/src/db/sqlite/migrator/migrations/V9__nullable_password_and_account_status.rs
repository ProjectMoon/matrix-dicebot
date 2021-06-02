pub fn migration() -> String {
    // sqlite does really support alter column, and barrel does not
    // implement the required workaround, so we do it ourselves!
    r#"
      CREATE TABLE IF NOT EXISTS "accounts2" (
         "user_id" TEXT PRIMARY KEY NOT NULL UNIQUE,
         "password" TEXT NULL,
         "account_status" TEXT NOT NULL CHECK(
             account_status IN ('not_registered', 'registered', 'awaiting_activation'
         ))
      );
      INSERT INTO accounts2 select *, 'registered' FROM accounts;
      DROP TABLE accounts;
      ALTER TABLE accounts2 RENAME TO accounts;
    "#
    .to_string()
}
