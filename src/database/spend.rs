use std::future::Future;
use std::io::Error;
use std::pin::Pin;
use surrealdb::engine::local::Db;
use surrealdb::sql::Thing;
use surrealdb::Surreal;
use crate::database::generate::DatabaseAsync;
use crate::database::global::{*};

fn get_proof_db_id(id: String, db: Surreal<Db>) -> Pin<Box<dyn Future<Output = Result<String, Error>>>>
{
    Box::pin(async move {
        let response: Option<Record> = db.select((RESOURCE, id.as_str())).await.expect(DB_ERROR_MSG);
        match response {
            Some(_) => Ok(format!("{}:{}", RESOURCE, id)),
            None => Err(Error::new(
                std::io::ErrorKind::Other,
                "Failed to retrieve from database".to_string(),
            )),
        }
    })
}


#[tokio::main]
pub async fn store_on_db(previous_proof_id: Option<String>, amount: i64, pointer: Option<String>, database: DatabaseAsync)
    -> Result<String, Error>
{
    match database.await {
        Ok(db) => {
            let id_result: Result<Option<String>, Error> = match previous_proof_id {
                None => Ok(None),
                Some(id) => {
                    match get_proof_db_id(id, db.clone()).await {
                        Ok(id) => Ok(Some(id)),
                        Err(error) => Err(error)
                    }
                }
            };
        
            match id_result {
                Ok(parent_id) => {

                    let result: Vec<ReputationProofDBWithId> = db
                            .query(
                                format!("SELECT * FROM reputation_proof WHERE pointer={}", 
                                pointer.clone().unwrap_or(String::from("")))
                            )
                            .await.expect(DB_ERROR_MSG)
                            .take(1).expect(DB_ERROR_MSG);

                    let raw_id: String = match &result[..] {

                        [_s] => {

                            let _updated: Option<Thing> = db
                            .update((RESOURCE, _s.id.as_str()))
                            .content(ReputationProofDB {
                                amount: amount + _s.amount,
                                pointer: pointer
                            })
                            .await.expect(DB_ERROR_MSG);

                            _s.id.clone()
                        },
                        _ => {
                            // Create a new person with a random id
                            let created: Vec<Record> = db
                                .create(RESOURCE)
                                .content(ReputationProofDB {
                                    pointer,
                                    amount  // TODO could check that amount <= proof->amount
                                })
                                .await.expect(DB_ERROR_MSG);
                
                            let raw_id = created.first().unwrap().id.to_string();
                
                            match parent_id {
                                None => {}
                                Some(parent_id) => {
                                    db.query(
                                        format!("RELATE {}->leaf->{}", parent_id, raw_id.to_string())
                                    )
                                    .await.expect(DB_ERROR_MSG);
                                }
                            }  

                            raw_id                 
                        }
                    };

                    let proof_id = raw_id.split_at((RESOURCE.to_owned()+":").len()).1.to_string();
                    Ok(proof_id)
                }
                Err(error) => Err(error)
            }
        },
        Err(err) => Err(err)
    }
}
