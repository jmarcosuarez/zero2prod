-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
    '5bfb7a12-20ae-43e2-8683-744af1f82978',
    'admin',
    '$argon2id$v=19$m=15000,t=2,p=1$bObqtzVmenZti94hpirc9Q$V6qrUfY/dRvcYexHILF4oLKZSzibM/py2FbVaaMYm2s'
);
