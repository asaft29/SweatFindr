INSERT INTO UTILIZATORI (email, parola, rol) VALUES
    ('admin@event.ro', 'parola_admin_hash', 'admin'),
    ('owner1@event.ro', 'parola_owner1_hash', 'owner-event'),
    ('owner2@event.ro', 'parola_owner2_hash', 'owner-event'),
    ('client@user.ro', 'parola_client_hash', 'client');


INSERT INTO EVENIMENTE (ID_OWNER, nume, locatie, descriere, numarLocuri) VALUES
    (2, 'Concert Vama Veche', 'Cluj-Napoca, BT Arena', 'Concert de muzică rock, gen: rock alternativ', 5000),
    (2, 'Festivalul de Teatru', 'Iași, Teatrul Național', 'Săptămâna dedicată pieselor de teatru clasic și modern', 1000),
    (3, 'Târg de Crăciun 2025', 'București, Piața Constituției', 'Târg anual de sărbători, decorațiuni și meșteșuguri', 20000),
    (3, 'Expoziție de Artă Modernă', 'Timișoara, Galeria Delta', 'Eveniment de o zi, cu picturi și sculpturi de artă contemporană', 500);


INSERT INTO PACHETE (ID_OWNER, nume, locatie, descriere) VALUES
    (2, 'Pachet Weekend Rock', 'Cluj-Napoca', 'Pachet ce include concertul și o petrecere post-eveniment.'),
    (2, 'Abonament Teatru 7 Zile', 'Iași', 'Abonament pentru toate cele 7 zile de festival.'),
    (3, 'Pachet București de Sărbătoare', 'București', 'Include intrarea la Târg și un voucher de discount.'),
    (3, 'Pachet Artă Complet', 'Timișoara', 'Intrare la expoziție și acces la catalogul digital de artă.');


INSERT INTO JOIN_PE (PachetID, EvenimentID, numarLocuri) VALUES
    (1, 1, 1000);

INSERT INTO JOIN_PE (PachetID, EvenimentID, numarLocuri) VALUES
    (2, 2, 300);

INSERT INTO JOIN_PE (PachetID, EvenimentID, numarLocuri) VALUES
    (3, 3, 5000);


INSERT INTO BILETE (COD, PachetID, EvenimentID) VALUES
    ('EVT-VAMA-001', NULL, 1),
    ('EVT-VAMA-002', NULL, 1),
    ('EVT-TEATRU-003', NULL, 2),

    ('PKT-ROCK-004', 1, 1),
    ('PKT-TEATRU-005', 2, 2),

    ('COD-006-INVALID', NULL, NULL);
