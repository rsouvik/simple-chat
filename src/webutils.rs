    use actix_web::{web, Responder, HttpResponse};
    //use crate::SwarmWebMessage;
    use tokio::{sync::mpsc};
    use serde::{Serialize, Deserialize};

    #[derive(Deserialize)]
    struct MyQueryParams {
        user_name: String,
    }

    //get handler
    pub(crate) async fn index(query: web::Query<MyQueryParams>) -> impl Responder {
        let name = &query.user_name;

        HttpResponse::Ok().body(format!("Data sent to libp2p swarm: {}", name))

        //let sender_clone = sender.get_ref().clone();

        // Send the query parameter data to the libp2p swarm through the sender channel
        /*if sender_clone.send(SwarmWebMessage::DataGet(name.to_owned())).await.is_ok() {
        HttpResponse::Ok().body(format!("Data sent to libp2p swarm: {}", name))
    } else {
        HttpResponse::InternalServerError().body("Failed to send data to libp2p swarm")
    }*/
    }

    pub(crate) async fn indexPost(query: web::Json<MyQueryParams>) -> impl Responder {
        /*let model_type = &query.mtype;
    let model_loc = &query.mlocation;
    let model_data_loc = &query.mdataloc;
    let model_algo = &query.malgo;
    let model_output_loc = &query.moutputloc;*/

        HttpResponse::Ok().body("Data sent to libp2p swarm")
        //let sender_clone = sender.get_ref().clone();

        /*let metadata = serde_json::json!(
                            {
                              "mtype": model_type,
                              "mlocation": model_loc,   //e.g. s3://picxelate/dgp/create_model.py
                              "mdataloc": model_data_loc,
                              "malgo": model_algo,
                              "moutputloc": model_output_loc,
                            }
                            );*/
        //e.g. s3://picxelate/dgp/create_model.py
        /*let metadata =     r#"{
                              "mtype": model_type,
                              "mlocation": model_loc,
                              "mdataloc": model_data_loc,
                              "malgo": model_algo,
                              "moutputloc": model_output_loc,
                            }"#;*/

        //do metadata check about known peers

        // Send the query parameter data to the libp2p swarm through the sender channel
        //if sender_clone.send(SwarmWebMessage::Data(model_loc.to_owned())).await.is_ok() {
        /*if sender_clone.send(SwarmWebMessage::Data(query)).await.is_ok() {
        //HttpResponse::Ok().body(format!("Data sent to libp2p swarm: {}", model_loc))
        HttpResponse::Ok().body("Data sent to libp2p swarm")

    } else {
        HttpResponse::InternalServerError().body("Failed to send data to libp2p swarm")
    }*/
    }
