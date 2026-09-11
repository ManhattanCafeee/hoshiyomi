-- 会话与刷新令牌改为只存 SHA-256 摘要（64 位十六进制）。
-- 旧行存的是原始 UUID，按摘要查询永远命中不到，直接清空；
-- 代价是升级后所有用户需重新登录一次。
DELETE FROM refresh_tokens;
DELETE FROM sessions;

ALTER TABLE sessions
    MODIFY COLUMN session_id VARCHAR(64) NOT NULL,
    ADD COLUMN last_activity DATETIME NOT NULL DEFAULT (UTC_TIMESTAMP());

-- 注意：RENAME COLUMN 与 MODIFY COLUMN 必须分成两条语句——
-- MySQL 在同一条 ALTER 里不会让 MODIFY 看到刚改名的新列名（报 1054 Unknown column）。
ALTER TABLE refresh_tokens RENAME COLUMN token TO token_hash;
ALTER TABLE refresh_tokens MODIFY COLUMN token_hash VARCHAR(64) NOT NULL;
