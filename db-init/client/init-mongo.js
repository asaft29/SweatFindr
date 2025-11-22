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
      lista_bilete: [],
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
      lista_bilete: [],
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
      lista_bilete: [],
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
      lista_bilete: [],
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
      lista_bilete: [],
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
      lista_bilete: [],
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
      lista_bilete: [],
    },
    {
      email: "gabriel.constantinescu@example.com",
      prenume: "Gabriel",
      nume: "Constantinescu",
      public_info: false,
      social_media: {
        public: false,
      },
      lista_bilete: [],
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
      lista_bilete: [],
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
