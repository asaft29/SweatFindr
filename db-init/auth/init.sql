DROP TABLE IF EXISTS UTILIZATORI CASCADE;

CREATE TABLE
    UTILIZATORI (
        ID SERIAL PRIMARY KEY,
        email VARCHAR(255) UNIQUE NOT NULL,
        parola VARCHAR(255) NOT NULL,
        rol VARCHAR(20) NOT NULL CHECK (rol IN ('admin', 'owner-event', 'client'))
    );