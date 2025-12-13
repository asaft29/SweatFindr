INSERT INTO
    UTILIZATORI (email, parola, rol, email_verified)
VALUES
    (
        'adminrusty@hotmail.co',
        '$2b$12$mhjNdOtFCwaqJKJojBTLkeG5Lg5BQoMpNytF1/Z6/DasiZ0d7xtKy',
        'admin',
        true
    ),
    (
        'clients-service',
        '$2b$12$SbgmUnJyc8.bUL.rQbIRG.u04YB..4c4k6hlr/vQynuaCjj6/bZqG',
        'clients-service',
        true
    ),
    (
        'eo_1@example.com',
        '$2b$12$mhjNdOtFCwaqJKJojBTLkeG5Lg5BQoMpNytF1/Z6/DasiZ0d7xtKy',
        'owner-event',
        true
    ),
    (
        'event-owner-2@example.com',
        '$2b$12$mhjNdOtFCwaqJKJojBTLkeG5Lg5BQoMpNytF1/Z6/DasiZ0d7xtKy',
        'owner-event',
        true
    ) ON CONFLICT (email) DO NOTHING;