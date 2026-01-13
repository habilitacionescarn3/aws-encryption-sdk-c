#![allow(clippy::boxed_local, reason = "need to pass Box<T> to destructors")]
use aws_config::{AppName, Region, SdkConfig};
use std::sync::LazyLock;

static DAFNY_TOKIO_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

#[cxx::bridge]
mod ffi {
    struct RetryConfig {
        mode_adaptive: bool,
        max_attempts: u32,
        initial_backoff_milli: u64,
        max_backoff_milli: u64,
        reconnect_all: bool,
        use_static_exponential_base: bool,
    }

    enum KmsConfigurationType {
        KmsKeyArn,
        KmsMrKeyArn,
        Discovery,
        MrDiscovery,
    }

    enum CacheType {
        NoCache,
        MultiThreadedCache,
    }

    struct MplAwsClientConfig {
        env: bool,
        name: String,
        region: String,
        retry: RetryConfig,
    }

    struct MultiThreadedCacheConfig {
        entryCapacity: u32,
        entryPruningTailSize: u32,
    }

    struct KeyStoreConfig {
        name: String,
        ddb_table_name: String,
        kms_configuration_type: KmsConfigurationType,
        kms_configuration_value: String,
        logical_key_store_name: String,
        id: String,
        grant_tokens: Vec<String>,
        ddb_client: *const MplDdbClient,
        kms_client: *const MplKmsClient,
    }

    struct HierarchicalKeyringInput {
        name: String,
        branch_key_id: String,
        key_store: *const KeyStore,
        ttl: u32,
        cache: CacheType,
        multi_threaded_cache: MultiThreadedCacheConfig,
        partition_id: String,
    }

    extern "Rust" {
        type MplDdbClient;
        fn create_ddb_client(value: &MplAwsClientConfig) -> Result<Box<MplDdbClient>>;
        fn delete_ddb_client(client: Box<MplDdbClient>) -> Result<()>;

        type KeyStore;
        fn create_keystore(value: &KeyStoreConfig) -> Result<Box<KeyStore>>;
        fn delete_keystore(client: Box<KeyStore>) -> Result<()>;

        type Keyring;
        fn create_hierarchical_keyring(value: &HierarchicalKeyringInput) -> Result<Box<Keyring>>;
        fn delete_keyring(client: Box<Keyring>) -> Result<()>;

        type MplKmsClient;
        fn create_kms_client(value: &MplAwsClientConfig) -> Result<Box<MplKmsClient>>;
        fn delete_kms_client(client: Box<MplKmsClient>) -> Result<()>;

        // fn default_retry_config() -> RetryConfig;
        fn default_client_config() -> MplAwsClientConfig;
        fn default_keystore_config() -> KeyStoreConfig;
        fn default_hierarchical_keyring_input() -> HierarchicalKeyringInput;
    }
}

#[allow(dead_code)]
struct MplKmsClient {
    name: String,
    client: aws_sdk_kms::Client,
}
#[allow(dead_code)]
struct MplDdbClient {
    name: String,
    client: aws_sdk_dynamodb::Client,
}

#[allow(dead_code)]
struct KeyStore {
    name: String,
    client: aws_mpl_legacy::deps::aws_cryptography_keyStore::client::Client,
}

#[allow(dead_code)]
struct Keyring {
    name: String,
    client: aws_mpl_legacy::types::keyring::KeyringRef,
}

fn default_hierarchical_keyring_input() -> ffi::HierarchicalKeyringInput {
    ffi::HierarchicalKeyringInput {
        name: String::default(),
        branch_key_id: String::default(),
        key_store: std::ptr::null(),
        ttl: 300,
        cache: ffi::CacheType::MultiThreadedCache,
        multi_threaded_cache: ffi::MultiThreadedCacheConfig {
            entryCapacity: 1000,
            entryPruningTailSize: 1,
        },
        partition_id: String::default(),
    }
}

fn default_keystore_config() -> ffi::KeyStoreConfig {
    ffi::KeyStoreConfig {
        name: String::default(),
        ddb_table_name: String::default(),
        kms_configuration_type: ffi::KmsConfigurationType::KmsKeyArn,
        kms_configuration_value: String::default(),
        logical_key_store_name: String::default(),
        id: String::default(),
        grant_tokens: Vec::default(),
        ddb_client: std::ptr::null(),
        kms_client: std::ptr::null(),
    }
}

fn default_client_config() -> ffi::MplAwsClientConfig {
    ffi::MplAwsClientConfig {
        env: true,
        name: "mpl aws client".to_string(),
        region: String::default(),
        retry: default_retry_config(),
    }
}

fn default_retry_config() -> ffi::RetryConfig {
    ffi::RetryConfig {
        mode_adaptive: false,
        max_attempts: 0,
        initial_backoff_milli: 0,
        max_backoff_milli: 0,
        reconnect_all: false,
        use_static_exponential_base: false,
    }
}

fn make_retry_config(config: &ffi::RetryConfig) -> aws_config::retry::RetryConfig {
    let mut out_config = if config.mode_adaptive {
        aws_config::retry::RetryConfig::adaptive()
    } else {
        aws_config::retry::RetryConfig::standard()
    };
    if config.max_attempts > 0 {
        out_config = out_config.with_max_attempts(config.max_attempts);
    }
    if config.initial_backoff_milli > 0 {
        out_config = out_config.with_initial_backoff(std::time::Duration::from_millis(
            config.initial_backoff_milli,
        ));
    }
    if config.max_backoff_milli > 0 {
        out_config =
            out_config.with_max_backoff(std::time::Duration::from_millis(config.max_backoff_milli));
    }
    if config.reconnect_all {
        out_config = out_config
            .with_reconnect_mode(aws_sdk_kms::config::retry::ReconnectMode::ReuseAllConnections);
    }
    if config.use_static_exponential_base {
        out_config = out_config.with_use_static_exponential_base(true);
    }
    out_config
}
fn delete_kms_client(_client: Box<MplKmsClient>) -> Result<(), String> {
    Ok(())
}
fn delete_ddb_client(_client: Box<MplDdbClient>) -> Result<(), String> {
    Ok(())
}
fn delete_keystore(_client: Box<KeyStore>) -> Result<(), String> {
    Ok(())
}
fn delete_keyring(_client: Box<Keyring>) -> Result<(), String> {
    Ok(())
}

fn make_cache_type(
    config: &ffi::HierarchicalKeyringInput,
) -> Result<aws_mpl_legacy::types::CacheType, String> {
    match config.cache {
        ffi::CacheType::NoCache => Ok(aws_mpl_legacy::types::CacheType::No(
            aws_mpl_legacy::types::NoCache::builder().build().unwrap(),
        )),
        ffi::CacheType::MultiThreadedCache => {
            let entry_capacity = config.multi_threaded_cache.entryCapacity;
            let entry_pruning_tail_size = config.multi_threaded_cache.entryPruningTailSize;
            Ok(aws_mpl_legacy::types::CacheType::MultiThreaded(
                aws_mpl_legacy::types::MultiThreadedCache::builder()
                    .entry_capacity(entry_capacity as i32)
                    .entry_pruning_tail_size(entry_pruning_tail_size as i32)
                    .build()
                    .unwrap(),
            ))
        }
        _ => Err("Invalid CacheType in HierarchicalKeyringInput".to_string()),
    }
}

fn make_kms_config(
    config: &ffi::KeyStoreConfig,
) -> Result<aws_mpl_legacy::deps::aws_cryptography_keyStore::types::KmsConfiguration, String> {
    match config.kms_configuration_type {
        ffi::KmsConfigurationType::KmsKeyArn => Ok(
            aws_mpl_legacy::deps::aws_cryptography_keyStore::types::KmsConfiguration::KmsKeyArn(
                config.kms_configuration_value.clone(),
            ),
        ),
        ffi::KmsConfigurationType::KmsMrKeyArn => Ok(
            aws_mpl_legacy::deps::aws_cryptography_keyStore::types::KmsConfiguration::KmsMrKeyArn(
                config.kms_configuration_value.clone(),
            ),
        ),
        ffi::KmsConfigurationType::Discovery => Ok(
            aws_mpl_legacy::deps::aws_cryptography_keyStore::types::KmsConfiguration::Discovery(
                aws_mpl_legacy::deps::aws_cryptography_keyStore::types::Discovery::builder()
                    .build()
                    .unwrap(),
            ),
        ),
        ffi::KmsConfigurationType::MrDiscovery => Ok(
            aws_mpl_legacy::deps::aws_cryptography_keyStore::types::KmsConfiguration::MrDiscovery(
                aws_mpl_legacy::deps::aws_cryptography_keyStore::types::MrDiscovery::builder()
                    .build()
                    .unwrap(),
            ),
        ),
        _ => Err("Invalid KmsConfigurationType".to_string()),
    }
}

fn create_hierarchical_keyring(
    input: &ffi::HierarchicalKeyringInput,
) -> Result<Box<Keyring>, String> {
    let mpl_config = aws_mpl_legacy::types::MaterialProvidersConfig::builder()
        .build()
        .unwrap();
    let mpl = aws_mpl_legacy::Client::from_conf(mpl_config).unwrap();
    let mut builder = mpl
        .create_aws_kms_hierarchical_keyring()
        .cache(make_cache_type(input)?)
        .branch_key_id(input.branch_key_id.clone())
        .ttl_seconds(input.ttl);
    if input.key_store.is_null() {
        return Err("key_store is null in create_hierarchical_keyring".to_string());
    } else {
        builder = builder.key_store(unsafe { (*input.key_store).client.clone() })
    }
    if !input.partition_id.is_empty() {
        builder = builder.partition_id(input.partition_id.clone());
    }

    let keyring = DAFNY_TOKIO_RUNTIME
        .block_on(builder.send())
        .map_err(|e| format!("{:?}", e))?;

    let keyring = Keyring {
        name: input.name.clone(),
        client: keyring,
    };
    Ok(Box::new(keyring))
}

fn create_keystore(input: &ffi::KeyStoreConfig) -> Result<Box<KeyStore>, String> {
    let mut builder = aws_mpl_legacy::deps::aws_cryptography_keyStore::types::key_store_config::KeyStoreConfig::builder();
    if input.kms_client.is_null() {
        return Err("kms_client is null in create_keystore".to_string());
    } else {
        builder = builder.kms_client(unsafe { (*input.kms_client).client.clone() });
    }
    if input.ddb_client.is_null() {
        return Err("ddb_client is null in create_keystore".to_string());
    } else {
        builder = builder.ddb_client(unsafe { (*input.ddb_client).client.clone() });
    }
    if input.ddb_table_name.is_empty() {
        return Err("ddb_table_name is empty in create_keystore".to_string());
    } else {
        builder = builder.ddb_table_name(input.ddb_table_name.clone());
    }
    if input.logical_key_store_name.is_empty() {
        return Err("logical_key_store_name is empty in create_keystore".to_string());
    } else {
        builder = builder.logical_key_store_name(input.logical_key_store_name.clone());
    }
    builder = builder.kms_configuration(make_kms_config(input)?);
    if !input.id.is_empty() {
        builder = builder.id(input.id.clone());
    }
    if !input.grant_tokens.is_empty() {
        builder = builder.grant_tokens(input.grant_tokens.clone());
    }
    let config = builder.build().map_err(|e| format!("{:?}", e))?;

    let store = aws_mpl_legacy::deps::aws_cryptography_keyStore::client::Client::from_conf(config)
        .map_err(|e| format!("{:?}", e))?;
    let store = KeyStore {
        name: input.name.clone(),
        client: store,
    };
    Ok(Box::new(store))
}

fn create_kms_client(input: &ffi::MplAwsClientConfig) -> Result<Box<MplKmsClient>, String> {
    let sdk_config = create_sdk_config(input.region.clone(), &input.retry);
    let client = aws_sdk_kms::Client::new(&sdk_config);
    let client = MplKmsClient {
        name: input.name.clone(),
        client,
    };
    Ok(Box::new(client))
}

fn create_ddb_client(input: &ffi::MplAwsClientConfig) -> Result<Box<MplDdbClient>, String> {
    let sdk_config = create_sdk_config(input.region.clone(), &input.retry);
    let client = aws_sdk_dynamodb::Client::new(&sdk_config);
    let client = MplDdbClient {
        name: input.name.clone(),
        client,
    };
    Ok(Box::new(client))
}

fn create_sdk_config(region: String, retry: &ffi::RetryConfig) -> SdkConfig {
    let shared_config = DAFNY_TOKIO_RUNTIME.block_on(aws_config::load_defaults(
        aws_config::BehaviorVersion::latest(),
    ));

    let user_agent_string = "AwsCryptographicMPL-C++-1.11.1";
    let current_app_name = shared_config
        .app_name()
        .map(|app_name| app_name.to_string())
        .unwrap_or_default();
    let new_app_name = if current_app_name.is_empty() {
        user_agent_string.to_string()
    } else {
        format!("{} {} ", current_app_name, user_agent_string)
    };
    let app_name = AppName::new(new_app_name).expect("Valid app name");
    let mut builder = shared_config
        .to_builder()
        .app_name(app_name)
        .retry_config(make_retry_config(retry));
    if !region.is_empty() {
        builder = builder.region(Region::new(region));
    }
    builder.build()
}
