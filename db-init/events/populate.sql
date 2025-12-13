TRUNCATE TABLE BILETE,
JOIN_PE,
PACHETE,
EVENIMENTE RESTART IDENTITY CASCADE;

INSERT INTO
    EVENIMENTE (ID_OWNER, nume, locatie, descriere, numarLocuri)
VALUES
    (
        3,
        'Concert Vama Veche',
        'Cluj-Napoca, BT Arena',
        'Concert de muzică rock alternativ cu trupa Vama Veche. Atmosphere electrizantă și hit-uri legendare!',
        5000
    ),
    (
        3,
        'Festival Electric Castle 2025',
        'Cluj, Domeniul Banffy',
        'Cel mai mare festival de muzică electronică din România cu artiști internaționali.',
        50000
    ),
    (
        3,
        'Concert Simfonic de Crăciun',
        'București, Sala Palatului',
        'Orchestra Filarmonică București prezintă concerte clasice de sărbători.',
        2500
    ),
    (
        4,
        'Untold Festival 2025',
        'Cluj-Napoca, Cluj Arena',
        'Festival internațional de muzică electronică, dans și cultură.',
        80000
    ),
    (
        4,
        'Festivalul de Teatru',
        'Iași, Teatrul Național',
        'Săptămâna dedicată pieselor de teatru clasic și modern. Reprezentații zilnice cu trupe din toată țara.',
        1000
    ),
    (
        4,
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
        3,
        'Târgul de Paște',
        'Brașov, Piața Sfatului',
        'Târg tradițional cu produse pascale, ouă decorate și muzică populară.',
        8000
    ),
    (
        3,
        'Maraton București 2025',
        'București, Piața Constituției',
        'Competiție sportivă internațională - maraton complet și semimaraton.',
        30000
    ),
    (
        3,
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
        3,
        'Festivalul Vinului și Bucatelor',
        'Alba Iulia, Cetatea Alba Carolina',
        'Degustări de vinuri românești premium și preparate gastronomice locale.',
        5000
    ),
    (
        3,
        'Expoziție de Artă Modernă',
        'Timișoara, Galeria Delta',
        'Colecție de artă contemporană: picturi, sculpturi și instalații multimedia.',
        500
    ),
    (
        4,
        'Bienala de Arhitectură',
        'București, MNAC',
        'Expoziție internațională dedicată arhitecturii contemporane și urbanismului.',
        2000
    ),
    (
        3,
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
        3,
        'Pachet Weekend Rock Cluj',
        'Cluj-Napoca',
        'Include Concert Vama Veche + Electric Castle cu acces VIP și transport inclus.',
        5000
    ),
    (
        3,
        'Abonament Muzical Complet',
        'Multiple',
        'Acces la toate concertele din Cluj și București pentru 2025.',
        2500
    ),
    (
        4,
        'Festival Pass Untold Premium',
        'Cluj-Napoca',
        'Abonament 4 zile Untold cu camping și early entry.',
        80000
    ),
    (
        4,
        'Abonament Teatru 7 Zile',
        'Iași',
        'Abonament pentru toate cele 7 zile de festival cu acces la toate reprezentațiile.',
        1000
    ),
    (
        4,
        'Pachet Cultură Sibiu',
        'Sibiu',
        'Spectacol Shakespeare + vizită muzeală ghidată.',
        450
    ),
    (
        4,
        'Art Lover Pass',
        'București',
        'Acces la Noaptea Albă + Bienala de Arhitectură + cataloguri digitale.',
        2000
    ),
    (
        4,
        'Pachet București de Sărbătoare',
        'București',
        'Include Târg de Crăciun, Concert Simfonic și voucher 20% discount la produse.',
        2500
    ),
    (
        4,
        'Experiență Medievală Completă',
        'Sighișoara',
        'Pachet 2 zile cu cazare, intrare festival și masă medievală.',
        15000
    ),
    (
        3,
        'Weekend Brașov Primăvară',
        'Brașov',
        'Târg de Paște + Escaladă spectatori cu cazare 2 nopți.',
        800
    ),
    (
        4,
        'Gourmet Experience',
        'Multiple',
        'Street Food Festival + Festival Vinului cu degustări premium.',
        5000
    ),
    (
        3,
        'Pachet Relaxare Alba Iulia',
        'Alba Iulia',
        'Festival Vinului cu tur ghidat și cazare spa.',
        5000
    ),
    (
        3,
        'Tech Enthusiast Bundle',
        'Cluj & Iași',
        'Tech Summit + Innovation Fest cu acces workshop-uri.',
        1500
    ),
    (
        4,
        'Future Innovation Pass',
        'Iași',
        'Innovation Fest cu demonstrații VR exclusive.',
        3000
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