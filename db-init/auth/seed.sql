INSERT INTO
    UTILIZATORI (email, parola, rol)
VALUES
    (
        'adminrusty@hotmail.co',
        '$2b$12$mhjNdOtFCwaqJKJojBTLkeG5Lg5BQoMpNytF1/Z6/DasiZ0d7xtKy',
        'admin'
    ) ON CONFLICT (email) DO NOTHING;