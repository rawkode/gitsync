// use super::GitSync;
// use actix::prelude::*;
// use git2::Repository;
// use git2::{Cred, RemoteCallbacks, ResetType};
// use log::info;
// use std::env;
// use std::fs;

// impl Actor for GitSync {
//     type Context = Context<Self>;

//     fn started(&mut self, ctx: &mut Context<Self>) {
//         info!("Starting ...");

//         // Clone Git repository
//         self.clone().clone_repository();

//         ctx.run_interval(self.sync_every, move |act, ctx| {
//             info!("Syncing ...");

//             // Prepare callbacks.
//             let mut callbacks = RemoteCallbacks::new();
//             callbacks.credentials(|_url, username_from_url, _allowed_types| {
//                 Cred::ssh_key(
//                     username_from_url.unwrap(),
//                     None,
//                     std::path::Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
//                     None,
//                 )
//             });

//             // Prepare fetch options.
//             let mut fo = git2::FetchOptions::new();
//             fo.remote_callbacks(callbacks);

//             let repository: Repository = match Repository::open("repository") {
//                 Ok(repository) => repository,
//                 Err(_) => {
//                     ctx.stop();
//                     panic!("Failed to open repository");
//                 }
//             };

//             let mut remote = match repository.find_remote("origin") {
//                 Ok(remote) => remote,
//                 Err(_) => {
//                     ctx.stop();
//                     panic!("Failed to get 'origin' remote from repository");
//                 }
//             };

//             let old_obj_id = match repository
//                 .refname_to_id(&format!("refs/remotes/{}/{}", "origin", "master"))
//             {
//                 Ok(r) => r,
//                 Err(_) => {
//                     ctx.stop();
//                     panic!("help");
//                 }
//             };

//             if let Err(e) = remote.fetch(&["master"], Some(&mut fo), None) {
//                 ctx.stop();
//                 panic!("Failed to fetch latest changes for repository: {}", e);
//             };

//             let new_obj_id = match repository
//                 .refname_to_id(&format!("refs/remotes/{}/{}", "origin", "master"))
//             {
//                 Ok(r) => r,
//                 Err(_) => {
//                     ctx.stop();
//                     panic!("help");
//                 }
//             };

//             if new_obj_id == old_obj_id {
//                 info!("No updates to repository");
//                 return;
//             }

//             let obj = match repository.find_object(new_obj_id, None) {
//                 Ok(r) => r,
//                 Err(_) => {
//                     ctx.stop();
//                     panic!("help 2");
//                 }
//             };

//             repository.reset(&obj, ResetType::Hard, None);

//             let paths = fs::read_dir("repository").unwrap();

//             for path in paths {
//                 println!("Name: {}", path.unwrap().path().display())
//             }

//             // info!("Sending RELOAD to NodeGroupController");
//             // act.cluster_controller.do_send(ReloadCommand);

//             info!("Synced!");
//         });
//     }
// }
