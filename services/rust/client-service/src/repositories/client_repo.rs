use bson::oid::ObjectId;
use bson::{Document, doc};
use futures::stream::TryStreamExt;
use mongodb::{Collection, Database};

use crate::models::client::{Client, ClientQuery, CreateClient, TicketRef, UpdateClient};
use crate::utils::error::ClientRepoError;

pub struct ClientRepo {
    collection: Collection<Client>,
}

impl ClientRepo {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("clients"),
        }
    }

    pub async fn check(&self) -> Result<(), ClientRepoError> {
        self.collection
            .count_documents(doc! {})
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub async fn list_clients(&self, query: ClientQuery) -> Result<Vec<Client>, ClientRepoError> {
        let mut filter = Document::new();

        if let Some(email) = query.email {
            filter.insert("email", doc! { "$regex": email, "$options": "i" });
        }

        if let Some(prenume) = query.prenume {
            filter.insert("prenume", doc! { "$regex": prenume, "$options": "i" });
        }

        if let Some(nume) = query.nume {
            filter.insert("nume", doc! { "$regex": nume, "$options": "i" });
        }

        let cursor = self
            .collection
            .find(filter)
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;

        let clients: Vec<Client> = cursor
            .try_collect()
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;

        Ok(clients)
    }

    pub async fn get_client(&self, id: &str) -> Result<Client, ClientRepoError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| ClientRepoError::InvalidObjectId(format!("Invalid ID: {}", id)))?;

        let client = self
            .collection
            .find_one(doc! { "_id": object_id })
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ClientRepoError::NotFound(format!("Client with id {} not found", id)))?;

        Ok(client)
    }

    pub async fn create_client(&self, client: CreateClient) -> Result<Client, ClientRepoError> {
        let existing = self
            .collection
            .find_one(doc! { "email": &client.email })
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            return Err(ClientRepoError::DuplicateEmail(format!(
                "Client with email {} already exists",
                client.email
            )));
        }

        let new_client = Client {
            id: ObjectId::new(),
            email: client.email,
            prenume: client.prenume,
            nume: client.nume,
            public_info: client.public_info,
            social_media: client.social_media,
            lista_bilete: vec![],
        };

        self.collection
            .insert_one(&new_client)
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;

        Ok(new_client)
    }

    pub async fn update_client(
        &self,
        id: &str,
        update: UpdateClient,
    ) -> Result<Client, ClientRepoError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| ClientRepoError::InvalidObjectId(format!("Invalid ID: {}", id)))?;

        self.get_client(id).await?;

        if let Some(ref email) = update.email {
            let existing = self
                .collection
                .find_one(doc! { "email": email, "_id": { "$ne": object_id } })
                .await
                .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;

            if existing.is_some() {
                return Err(ClientRepoError::DuplicateEmail(format!(
                    "Mail {} is already in use!",
                    email
                )));
            }
        }

        let mut update_doc = Document::new();

        if let Some(email) = update.email {
            update_doc.insert("email", email);
        }
        if let Some(prenume) = update.prenume {
            update_doc.insert("prenume", prenume);
        }
        if let Some(nume) = update.nume {
            update_doc.insert("nume", nume);
        }
        if let Some(public_info) = update.public_info {
            update_doc.insert("public_info", public_info);
        }
        if let Some(social_media) = update.social_media {
            update_doc.insert("social_media", bson::to_bson(&social_media).unwrap());
        }

        self.collection
            .update_one(doc! { "_id": object_id }, doc! { "$set": update_doc })
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;

        self.get_client(id).await
    }

    pub async fn delete_client(&self, id: &str) -> Result<(), ClientRepoError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| ClientRepoError::InvalidObjectId(format!("Invalid ID: {}", id)))?;

        let result = self
            .collection
            .delete_one(doc! { "_id": object_id })
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;

        if result.deleted_count == 0 {
            return Err(ClientRepoError::NotFound(format!(
                "Client with id {} not found",
                id
            )));
        }

        Ok(())
    }

    pub async fn get_client_tickets(&self, id: &str) -> Result<Vec<TicketRef>, ClientRepoError> {
        let client = self.get_client(id).await?;
        Ok(client.lista_bilete)
    }

    pub async fn add_ticket_ref_to_client(
        &self,
        id: &str,
        ticket_ref: TicketRef,
    ) -> Result<Client, ClientRepoError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| ClientRepoError::InvalidObjectId(format!("Invalid ID: {}", id)))?;

        let client = self.get_client(id).await?;

        if client.lista_bilete.iter().any(|t| t.cod == ticket_ref.cod) {
            tracing::warn!(
                "Ticket {} already exists in client {}. Skipping duplicate.",
                ticket_ref.cod,
                id
            );
            return Ok(client);
        }

        self.collection
            .update_one(
                doc! { "_id": object_id },
                doc! { "$push": { "lista_bilete": bson::to_bson(&ticket_ref).unwrap() } },
            )
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;

        self.get_client(id).await
    }

    pub async fn remove_ticket_from_client(
        &self,
        id: &str,
        ticket_cod: &str,
    ) -> Result<Client, ClientRepoError> {
        let object_id = ObjectId::parse_str(id)
            .map_err(|_| ClientRepoError::InvalidObjectId(format!("Invalid ID: {}", id)))?;

        self.get_client(id).await?;

        self.collection
            .update_one(
                doc! { "_id": object_id },
                doc! { "$pull": { "lista_bilete": { "cod": ticket_cod } } },
            )
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;

        self.get_client(id).await
    }

    pub async fn find_client_by_ticket_code(
        &self,
        ticket_code: &str,
    ) -> Result<Option<Client>, ClientRepoError> {
        let client = self
            .collection
            .find_one(doc! { "lista_bilete.cod": ticket_code })
            .await
            .map_err(|e| ClientRepoError::DatabaseError(e.to_string()))?;

        Ok(client)
    }
}
