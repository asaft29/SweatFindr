DROP TABLE IF EXISTS EVENIMENTE CASCADE;

DROP TABLE IF EXISTS PACHETE CASCADE;

DROP TABLE IF EXISTS JOIN_PE CASCADE;

DROP TABLE IF EXISTS BILETE CASCADE;

CREATE EXTENSION IF NOT EXISTS unaccent;

CREATE TABLE
    EVENIMENTE (
        ID SERIAL PRIMARY KEY,
        ID_OWNER INTEGER NOT NULL,
        nume VARCHAR(255) UNIQUE NOT NULL,
        locatie VARCHAR(255) NULL,
        descriere TEXT NULL,
        numarLocuri INTEGER NULL
    );

CREATE TABLE
    PACHETE (
        ID SERIAL PRIMARY KEY,
        ID_OWNER INTEGER NOT NULL,
        nume VARCHAR(255) UNIQUE NOT NULL,
        locatie VARCHAR(255) NULL,
        descriere TEXT NULL,
        numarLocuri INTEGER NULL
    );

CREATE TABLE
    JOIN_PE (
        PachetID INTEGER REFERENCES PACHETE (ID) ON DELETE CASCADE,
        EvenimentID INTEGER REFERENCES EVENIMENTE (ID) ON DELETE CASCADE,
        PRIMARY KEY (PachetID, EvenimentID)
    );

CREATE TABLE
    BILETE (
        COD VARCHAR(50) PRIMARY KEY,
        PachetID INTEGER REFERENCES PACHETE (ID) ON DELETE SET NULL,
        EvenimentID INTEGER REFERENCES EVENIMENTE (ID) ON DELETE SET NULL,
        CONSTRAINT chk_bilet_exclusiv CHECK (
            (
                PachetID IS NOT NULL
                AND EvenimentID IS NULL
            )
            OR (
                PachetID IS NULL
                AND EvenimentID IS NOT NULL
            )
        )
    );

CREATE TABLE
    REFUND_REQUESTS (
        id SERIAL PRIMARY KEY,
        ticket_cod VARCHAR(50) NOT NULL,
        requester_id INTEGER NOT NULL,
        requester_email VARCHAR(255) NOT NULL,
        event_id INTEGER REFERENCES EVENIMENTE (ID) ON DELETE SET NULL,
        packet_id INTEGER REFERENCES PACHETE (ID) ON DELETE SET NULL,
        event_owner_id INTEGER NOT NULL,
        status VARCHAR(20) DEFAULT 'PENDING' NOT NULL,
        reason TEXT,
        rejection_message TEXT,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        resolved_at TIMESTAMP
    );