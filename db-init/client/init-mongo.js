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
  print("Created collection: clients");
} else {
  print("Collection clients already exists");
}

db.clients.createIndex({ email: 1 }, { unique: true });
db.clients.createIndex({ prenume: 1 });
db.clients.createIndex({ nume: 1 });
db.clients.createIndex({ "lista_bilete.cod": 1 });

print("Created indexes on clients collection");
print("Database setup complete");
