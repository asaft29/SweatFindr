INSERT INTO
    UTILIZATORI (email, parola, rol)
VALUES
    (
        'adminrusty@hotmail.co',
        '$2b$12$mhjNdOtFCwaqJKJojBTLkeG5Lg5BQoMpNytF1/Z6/DasiZ0d7xtKy',
        'admin'
    ),
    (
        'clients-service',
        '$2b$12$SbgmUnJyc8.bUL.rQbIRG.u04YB..4c4k6hlr/vQynuaCjj6/bZqG',
        'clients-service'
    ) ON CONFLICT (email) DO NOTHING;