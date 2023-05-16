use std::{error::Error, str};
use std::env;

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use s3::BucketConfiguration;

fn main() -> Result<(), Box<dyn Error>> {
    let key = env::args().nth(1).expect("Minio key-path not provided");
    
    let bucket_name = "lexicstack"

    let mut base_url = BaseUrl::from_string("http://localhost").unwrap();
    // base_url.https = true;

    let static_provider = StaticProvider::new(
        "Q3AM3UQ867SPQQA43P2F",
        "zuf+tfteSlswRu7BJ86wekitnifILbZam1KYY3TG",
        None,
    );

    let mut client = Client::new(base_url.clone(), Some(static_provider));
    
    // Check 'lexicstack' bucket exist or not.
    let exists = client
        .bucket_exists(&BucketExistsArgs::new(&bucket_name).unwrap())
        .await
        .unwrap();
    
    let (data, _) = bucket.get_object(key).await?;
	
    let bucket = Bucket::new_with_path_style(
		bucket_name,
		Region::Custom {
			region: "".to_owned(),
			endpoint: "http://127.0.0.1:9000".to_owned(),
		},
		Credentials {
			access_key: Some("".to_owned()),
			secret_key: Some("".to_owned()),
			security_token: None,
			session_token: None,
		},
	)?;

	let (_, code) = bucket.get_object(key).await?;
	
    if code == 404 {
        println!("=== Bucket created\n{} - {} - {}");
	}

	Ok(())
}