TRUNCATE TABLE BILETE,
JOIN_PE,
PACHETE,
EVENIMENTE,
UTILIZATORI RESTART IDENTITY CASCADE;

INSERT INTO
    UTILIZATORI (email, parola, rol)
VALUES
    (
        'admin@eventmaster.ro',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7BvQ8bHqP6',
        'admin'
    ),
    (
        'ion.popescu@events.ro',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7BvQ8bHqP6',
        'owner-event'
    ),
    (
        'maria.ionescu@culture.ro',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7BvQ8bHqP6',
        'owner-event'
    ),
    (
        'andrei.gheorghe@promotions.ro',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7BvQ8bHqP6',
        'owner-event'
    ),
    (
        'elena.stan@artevents.ro',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7BvQ8bHqP6',
        'owner-event'
    ),
    (
        'client1@gmail.com',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7BvQ8bHqP6',
        'client'
    ),
    (
        'client2@yahoo.com',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7BvQ8bHqP6',
        'client'
    ),
    (
        'ana.popescu@outlook.com',
        '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5NU7BvQ8bHqP6',
        'client'
    );

INSERT INTO
    EVENIMENTE (ID_OWNER, nume, locatie, descriere, numarLocuri)
VALUES
    (
        2,
        'Concert Vama Veche',
        'Cluj-Napoca, BT Arena',
        'Concert de muzică rock alternativ cu trupa Vama Veche. Atmosphere electrizantă și hit-uri legendare!',
        5000
    ),
    (
        2,
        'Festival Electric Castle 2025',
        'Cluj, Domeniul Banffy',
        'Cel mai mare festival de muzică electronică din România cu artiști internaționali.',
        50000
    ),
    (
        2,
        'Concert Simfonic de Crăciun',
        'București, Sala Palatului',
        'Orchestra Filarmonică București prezintă concerte clasice de sărbători.',
        2500
    ),
    (
        3,
        'Untold Festival 2025',
        'Cluj-Napoca, Cluj Arena',
        'Festival internațional de muzică electronică, dans și cultură.',
        80000
    ),
    (
        3,
        'Festivalul de Teatru',
        'Iași, Teatrul Național',
        'Săptămâna dedicată pieselor de teatru clasic și modern. Reprezentații zilnice cu trupe din toată țara.',
        1000
    ),
    (
        3,
        'Spectacol Shakespeare',
        'Sibiu, Teatrul Radu Stanca',
        'Adaptare modernă a piesei "Hamlet" de către regizorul Ion Caramitru.',
        450
    ),
    (
        4,
        'Noaptea Albă a Galeriilor',
        'București, Centrul Vechi',
        'Eveniment cultural cu expoziții de artă contemporană în 30+ galerii.',
        10000
    ),
    (
        4,
        'Târg de Crăciun 2025',
        'București, Piața Constituției',
        'Târg anual de sărbători cu decorațiuni handmade, meșteșuguri tradiționale și delicii culinare.',
        20000
    ),
    (
        4,
        'Festivalul Medieval Sighișoara',
        'Sighișoara, Cetate',
        'Reconstituire medievală cu cavaleri, meșteșugari și spectacole de epocă.',
        15000
    ),
    (
        5,
        'Târgul de Paște',
        'Brașov, Piața Sfatului',
        'Târg tradițional cu produse pascale, ouă decorate și muzică populară.',
        8000
    ),
    (
        2,
        'Maraton București 2025',
        'București, Piața Constituției',
        'Competiție sportivă internațională - maraton complet și semimaraton.',
        30000
    ),
    (
        5,
        'Cupa României la Escaladă',
        'Brașov, Sala Sporturilor',
        'Competiție națională de escaladă sportivă pentru toate categoriile de vârstă.',
        800
    ),
    (
        4,
        'Street Food Festival',
        'Timișoara, Piața Victoriei',
        'Festival culinar cu food trucks, cuisine internațională și muzică live.',
        12000
    ),
    (
        5,
        'Festivalul Vinului și Bucatelor',
        'Alba Iulia, Cetatea Alba Carolina',
        'Degustări de vinuri românești premium și preparate gastronomice locale.',
        5000
    ),
    (
        5,
        'Expoziție de Artă Modernă',
        'Timișoara, Galeria Delta',
        'Colecție de artă contemporană: picturi, sculpturi și instalații multimedia.',
        500
    ),
    (
        3,
        'Bienala de Arhitectură',
        'București, MNAC',
        'Expoziție internațională dedicată arhitecturii contemporane și urbanismului.',
        2000
    ),
    (
        2,
        'Tech Summit România 2025',
        'Cluj-Napoca, Grand Hotel Italia',
        'Conferință de tehnologie cu speakeri internaționali, workshop-uri AI și networking.',
        1500
    ),
    (
        4,
        'Innovation Fest',
        'Iași, Palas Mall',
        'Expoziție de startup-uri, roboti, VR/AR și tehnologii emergente.',
        3000
    );

INSERT INTO
    PACHETE (ID_OWNER, nume, locatie, descriere, numarLocuri)
VALUES
    (
        2,
        'Pachet Weekend Rock Cluj',
        'Cluj-Napoca',
        'Include Concert Vama Veche + Electric Castle cu acces VIP și transport inclus.',
        800
    ),
    (
        2,
        'Abonament Muzical Complet',
        'Multiple',
        'Acces la toate concertele din Cluj și București pentru 2025.',
        400
    ),
    (
        3,
        'Festival Pass Untold Premium',
        'Cluj-Napoca',
        'Abonament 4 zile Untold cu camping și early entry.',
        5000
    ),
    (
        3,
        'Abonament Teatru 7 Zile',
        'Iași',
        'Abonament pentru toate cele 7 zile de festival cu acces la toate reprezentațiile.',
        300
    ),
    (
        3,
        'Pachet Cultură Sibiu',
        'Sibiu',
        'Spectacol Shakespeare + vizită muzeală ghidată.',
        150
    ),
    (
        4,
        'Art Lover Pass',
        'București',
        'Acces la Noaptea Albă + Bienala de Arhitectură + cataloguri digitale.',
        400
    ),
    (
        4,
        'Pachet București de Sărbătoare',
        'București',
        'Include Târg de Crăciun, Concert Simfonic și voucher 20% discount la produse.',
        600
    ),
    (
        4,
        'Experiență Medievală Completă',
        'Sighișoara',
        'Pachet 2 zile cu cazare, intrare festival și masă medievală.',
        3000
    ),
    (
        5,
        'Weekend Brașov Primăvară',
        'Brașov',
        'Târg de Paște + Escaladă spectatori cu cazare 2 nopți.',
        200
    ),
    (
        4,
        'Gourmet Experience',
        'Multiple',
        'Street Food Festival + Festival Vinului cu degustări premium.',
        1500
    ),
    (
        5,
        'Pachet Relaxare Alba Iulia',
        'Alba Iulia',
        'Festival Vinului cu tur ghidat și cazare spa.',
        1000
    ),
    (
        2,
        'Tech Enthusiast Bundle',
        'Cluj & Iași',
        'Tech Summit + Innovation Fest cu acces workshop-uri.',
        500
    ),
    (
        4,
        'Future Innovation Pass',
        'Iași',
        'Innovation Fest cu demonstrații VR exclusive.',
        600
    );

INSERT INTO
    JOIN_PE (PachetID, EvenimentID)
VALUES
    (1, 1),
    (1, 2),
    (2, 1),
    (2, 2),
    (2, 3),
    (3, 4),
    (4, 5),
    (5, 6),
    (6, 7),
    (6, 16),
    (7, 8),
    (7, 3),
    (8, 9),
    (9, 10),
    (9, 12),
    (10, 13),
    (10, 14),
    (11, 14),
    (12, 17),
    (12, 18),
    (13, 18);

INSERT INTO
    BILETE (COD, PachetID, EvenimentID)
VALUES
    ('EVT-VAMA-2025-001', NULL, 1),
    ('EVT-VAMA-2025-002', NULL, 1),
    ('EVT-VAMA-2025-003', NULL, 1),
    ('EVT-ELECTRIC-2025-001', NULL, 2),
    ('EVT-ELECTRIC-2025-002', NULL, 2),
    ('EVT-TEATRU-IASI-001', NULL, 5),
    ('EVT-TEATRU-IASI-002', NULL, 5),
    ('EVT-UNTOLD-VIP-001', NULL, 4),
    ('EVT-UNTOLD-VIP-002', NULL, 4),
    ('EVT-UNTOLD-VIP-003', NULL, 4),
    ('EVT-CRACIUN-BUC-001', NULL, 8),
    ('EVT-CRACIUN-BUC-002', NULL, 8),
    ('EVT-MEDIEVAL-SGH-001', NULL, 9),
    ('EVT-STREETFOOD-TM-001', NULL, 13),
    ('EVT-MARATON-BUC-001', NULL, 11),
    ('EVT-TECHSUMMIT-CLJ-001', NULL, 17),
    ('PKT-ROCK-WEEKEND-001', 1, NULL),
    ('PKT-ROCK-WEEKEND-002', 1, NULL),
    ('PKT-ROCK-WEEKEND-003', 1, NULL),
    ('PKT-TEATRU-7ZILE-001', 4, NULL),
    ('PKT-TEATRU-7ZILE-002', 4, NULL),
    ('PKT-UNTOLD-PREM-001', 3, NULL),
    ('PKT-UNTOLD-PREM-002', 3, NULL),
    ('PKT-SAARBATOARE-BUC-001', 7, NULL),
    ('PKT-SAARBATOARE-BUC-002', 7, NULL),
    ('PKT-MEDIEVAL-FULL-001', 8, NULL),
    ('PKT-GOURMET-EXP-001', 10, NULL),
    ('PKT-GOURMET-EXP-002', 10, NULL),
    ('PKT-TECH-BUNDLE-001', 12, NULL),
    ('PKT-TECH-BUNDLE-002', 12, NULL);