// use std::path::PathBuf;

// use tokio::net::UnixListener;

// use crate::config::admin::AdminServerConfig;

// pub struct AdminServer {
    
// }

// impl AdminServer {
//     pub fn run(
//         config: AdminServerConfig
//     ) -> Self { 

//         let listener = UnixListener::bind("/tmp/warp.sock").unwrap();
//         let incoming = UnixListenerStream::new(listener);
//         warp::serve(warp::fs::dir("examples/dir"))
//             .run_incoming(incoming)
//             .await;


//         Self { 
//             sock
//         }
//     }
// }
