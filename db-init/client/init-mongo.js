db = db.getSiblingDB("clientsdb");

try {
  db.createUser({
    user: "clientuser",
    pwd: "clientpass",
    roles: [{ role: "readWrite", db: "clientsdb" }],
  });
  print("Created user: clientuser");
} catch (e) {
  if (e.code === 51003) {
    print("User clientuser already exists");
  } else {
    throw e;
  }
}

if (!db.getCollectionNames().includes("clients")) {
  db.createCollection("clients");
}

if (db.clients.countDocuments() === 0) {
  db.clients.insertMany([
    {
      email: "ion.popescu@example.com",
      prenume: "Ion",
      nume: "Popescu",
      public_info: true,
      social_media: {
        facebook: "https://facebook.com/ion.popescu",
        twitter: "https://twitter.com/ionpopescu",
        instagram: "https://instagram.com/ion.popescu",
        public: true,
      },
      lista_bilete: [
        {
          cod: "TCK-2024-001",
          nume_eveniment: "Tech Conference 2024",
          locatie: "Iasi, Romania",
        },
        {
          cod: "TCK-2024-002",
          nume_eveniment: "Web Summit",
          locatie: "Lisbon, Portugal",
        },
      ],
    },
    {
      email: "maria.ionescu@example.com",
      prenume: "Maria",
      nume: "Ionescu",
      public_info: true,
      social_media: {
        linkedin: "https://linkedin.com/in/maria-ionescu",
        twitter: "https://twitter.com/maria_ionescu",
        public: true,
      },
      lista_bilete: [
        {
          cod: "TCK-2024-003",
          nume_eveniment: "DevOps Days",
          locatie: "Bucharest, Romania",
        },
      ],
    },
    {
      email: "andrei.stanescu@example.com",
      prenume: "Andrei",
      nume: "Stanescu",
      public_info: false,
      social_media: {
        linkedin: "https://linkedin.com/in/andrei-stanescu",
        public: false,
      },
      lista_bilete: [],
    },
    {
      email: "elena.georgescu@example.com",
      prenume: "Elena",
      nume: "Georgescu",
      public_info: true,
      social_media: {
        facebook: "https://facebook.com/elena.georgescu",
        instagram: "https://instagram.com/elena.georgescu",
        public: true,
      },
      lista_bilete: [
        {
          cod: "EVT-VAMA-2025-001",
          nume_eveniment: "Concert Vama Veche",
          locatie: "Cluj-Napoca, BT Arena",
        },
        {
          cod: "EVT-ELECTRIC-2025-001",
          nume_eveniment: "Festival Electric Castle 2025",
          locatie: "Cluj, Domeniul Banffy",
        },
        {
          cod: "PKT-ROCK-WEEKEND-001",
          nume_eveniment: "Pachet Weekend Rock Cluj",
          locatie: "Cluj-Napoca",
        },
      ],
    },
    {
      email: "vlad.popa@example.com",
      prenume: "Vlad",
      nume: "Popa",
      public_info: true,
      social_media: {
        twitter: "https://twitter.com/vladpopa",
        linkedin: "https://linkedin.com/in/vlad-popa",
        public: true,
      },
      lista_bilete: [
        {
          cod: "EVT-UNTOLD-VIP-001",
          nume_eveniment: "Untold Festival 2025",
          locatie: "Cluj-Napoca, Cluj Arena",
        },
      ],
    },
    {
      email: "diana.marin@example.com",
      prenume: "Diana",
      nume: "Marin",
      public_info: false,
      social_media: {
        instagram: "https://instagram.com/diana.marin.private",
        public: false,
      },
      lista_bilete: [
        {
          cod: "EVT-TEATRU-IASI-001",
          nume_eveniment: "Festivalul de Teatru",
          locatie: "Iași, Teatrul Național",
        },
        {
          cod: "PKT-TEATRU-7ZILE-001",
          nume_eveniment: "Abonament Teatru 7 Zile",
          locatie: "Iași",
        },
      ],
    },
    {
      email: "cosmin.dumitrescu@example.com",
      prenume: "Cosmin",
      nume: "Dumitrescu",
      public_info: true,
      social_media: {
        facebook: "https://facebook.com/cosmin.dumitrescu",
        twitter: "https://twitter.com/cosmindum",
        linkedin: "https://linkedin.com/in/cosmin-dumitrescu",
        public: true,
      },
      lista_bilete: [
        {
          cod: "EVT-TECHSUMMIT-CLJ-001",
          nume_eveniment: "Tech Summit România 2025",
          locatie: "Cluj-Napoca, Grand Hotel Italia",
        },
        {
          cod: "PKT-TECH-BUNDLE-001",
          nume_eveniment: "Tech Enthusiast Bundle",
          locatie: "Cluj & Iași",
        },
      ],
    },
    {
      email: "alexandra.radu@example.com",
      prenume: "Alexandra",
      nume: "Radu",
      public_info: true,
      social_media: {
        instagram: "https://instagram.com/alexandra.radu",
        facebook: "https://facebook.com/alexandra.radu",
        public: true,
      },
      lista_bilete: [
        {
          cod: "EVT-CRACIUN-BUC-001",
          nume_eveniment: "Târg de Crăciun 2025",
          locatie: "București, Piața Constituției",
        },
        {
          cod: "PKT-SAARBATOARE-BUC-001",
          nume_eveniment: "Pachet București de Sărbătoare",
          locatie: "București",
        },
      ],
    },
    {
      email: "gabriel.constantinescu@example.com",
      prenume: "Gabriel",
      nume: "Constantinescu",
      public_info: false,
      social_media: {
        public: false,
      },
      lista_bilete: [
        {
          cod: "EVT-MARATON-BUC-001",
          nume_eveniment: "Maraton București 2025",
          locatie: "București, Piața Constituției",
        },
      ],
    },
    {
      email: "mihaela.stoica@example.com",
      prenume: "Mihaela",
      nume: "Stoica",
      public_info: true,
      social_media: {
        linkedin: "https://linkedin.com/in/mihaela-stoica",
        twitter: "https://twitter.com/mihaelastoica",
        instagram: "https://instagram.com/mihaela_stoica",
        public: true,
      },
      lista_bilete: [
        {
          cod: "EVT-MEDIEVAL-SGH-001",
          nume_eveniment: "Festivalul Medieval Sighișoara",
          locatie: "Sighișoara, Cetate",
        },
        {
          cod: "EVT-STREETFOOD-TM-001",
          nume_eveniment: "Street Food Festival",
          locatie: "Timișoara, Piața Victoriei",
        },
        {
          cod: "PKT-GOURMET-EXP-001",
          nume_eveniment: "Gourmet Experience",
          locatie: "Multiple",
        },
      ],
    },
  ]);
  print("Inserted 10 sample clients with various ticket configurations");
} else {
  print("Clients collection already has data, skipping insert");
}

db.clients.createIndex({ email: 1 }, { unique: true });
db.clients.createIndex({ prenume: 1 });
db.clients.createIndex({ nume: 1 });
db.clients.createIndex({ "lista_bilete.cod": 1 });

var stats = {
  totalClients: db.clients.countDocuments(),
  clientsWithTickets: db.clients.countDocuments({
    "lista_bilete.0": { $exists: true },
  }),
  clientsWithoutTickets: db.clients.countDocuments({ lista_bilete: [] }),
  publicProfiles: db.clients.countDocuments({ public_info: true }),
  privateProfiles: db.clients.countDocuments({ public_info: false }),
};

print(JSON.stringify(stats, null, 2));
