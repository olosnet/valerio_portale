use app_modules::{
    astronomia::{
        oggetti_astronomici::OggettiAstronomiciModule,
        sessioni_osservative::SessioniOsservativeModule,
        siti_osservativi::SitiOsservativiModule,
    },
    base::{
        enums::EnumsModule, filemanager::FileManagerModule,
        filemanager_images::FileManagerImagesModule, groups::GroupsModule, users::UsersModule,
    },
};
use clap::{Arg, Command};
use cornetti::mongo::{
    confs::MongoDBConfig, helpers::init_mongo_modules, services::MongoDBService,
    traits::MongoBaseModule,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut command: Command = Command::new("app_managment")
        .about("Managment API Application")
        .arg(
            Arg::new("register-modules")
                .short('r')
                .long("register-modules")
                .help("Run module registration")
                .action(clap::ArgAction::SetTrue),
        );

    let matches = command.clone().get_matches();

    if matches.args_present() {
        if matches.get_flag("register-modules") {
            log::info!("Register modules...");

            let mongo_config: MongoDBConfig = MongoDBConfig::from_env();
            let mongo_connection = MongoDBService::new(&mongo_config).await.unwrap();

            init_mongo_modules(&mongo_connection).await.unwrap();
            UsersModule::register(&mongo_connection).await.unwrap();
            GroupsModule::register(&mongo_connection).await.unwrap();
            EnumsModule::register(&mongo_connection).await.unwrap();
            OggettiAstronomiciModule::register(&mongo_connection)
                .await
                .unwrap();
            SitiOsservativiModule::register(&mongo_connection)
                .await
                .unwrap();
            SessioniOsservativeModule::register(&mongo_connection)
                .await
                .unwrap();
            FileManagerModule::register(&mongo_connection)
                .await
                .unwrap();
            FileManagerImagesModule::register(&mongo_connection)
                .await
                .unwrap();

            log::info!("All modules registered successfully.");
        }
    } else {
        command.print_help().unwrap();
    }
}
