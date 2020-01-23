extern crate rusoto_core;
extern crate rusoto_s3;
extern crate time;

use std::env;
use std::str;
use time::Time;

// TODO review how credentials are looked up by default
use rusoto_core::{Region, RusotoError};
use rusoto_s3::{
    S3, S3Client, CreateBucketRequest, ListObjectsV2Request, DeleteBucketRequest
};

struct S3Demo {
    region: Region,
    s3: S3Client,
    bucket_name: String,
    // This flag signifies whether this bucket was already deleted as part of a test
    bucket_deleted: bool,
}

impl S3Demo {
    // construct a new S3 demo struct
    fn new(bucket_name: String) -> S3Demo {
        let region = if let Ok(endpoint) = env::var("S3_ENDPOINT") {
            let region = Region::Custom {
                name: "us-east-1".to_owned(),
                endpoint: endpoint.to_owned(),
            };
            println!(
                "picked up non-standard endpoint {:?} from S3_ENDPOINT env. variable",
                region
            );
            region
        } else {
            Region::UsEast1
        };

        S3Demo {
            region: region.to_owned(),
            s3: S3Client::new(region),
            bucket_name: bucket_name.to_owned(),
            bucket_deleted: false,
        }
    }

    // TODO should there be more traits implemented for the S3Demo?
}

fn delete_demo_bucket(client: &S3Client, bucket: &str) {
    let delete_bucket_req = DeleteBucketRequest {
        bucket: bucket.to_owned(),
        ..Default::default()
    };

    let result = client.delete_bucket(delete_bucket_req).sync();
    println!("{:#?}", result);
    match result {
        Err(e) => match e {
            RusotoError::Unknown(ref e) => panic!(
                "Couldn't delete bucket because: {}",
                str::from_utf8(&e.body).unwrap()
            ),
            _ => panic!("Error from S3 different than expected"),
        },
        Ok(_) => (),
    }
}

// creates a bucket and test listing buckets and items in bucket
fn s3_demo_creation_deletion() {

    let bucket_name = format!("rust-devops-webinar-demo-bucket-{}", Time::now().second());
    let mut demo = S3Demo::new(bucket_name.clone());

    let create_bucket_req = CreateBucketRequest {
        bucket: bucket_name.clone(),
        ..Default::default()
    };

    // first create a bucket
    let create_bucket_resp = demo.s3.create_bucket(create_bucket_req).sync();
    assert!(create_bucket_resp.is_ok());
    println!(
        "Bucket {} created\n response:\n {:#?}",
        bucket_name.clone(),
        create_bucket_resp.unwrap()
    );

    // TODO add an item
    // TODO and break this and listing the items into a different function
    // that we then call here like we eventually do with deletion.
    // Or rig these up with CLI options with CLAP

    // now lets check for our bucket and list items in the one we created
    let resp = demo.s3.list_buckets().sync();
    assert!(resp.is_ok());

    let resp = resp.unwrap();
    let mut bucket_found = false;
    for bucket in resp.buckets.unwrap().iter() {
        if bucket.name == Some(bucket_name.clone()) {
            bucket_found = true;
            break;
        }
    }
    assert!(bucket_found);

    let list_obj_req = ListObjectsV2Request {
        bucket: bucket_name.to_owned(),
        start_after: Some("foo".to_owned()), // TODO
        ..Default::default()
    };
    let result = demo.s3.list_objects_v2(list_obj_req).sync();
    assert!(result.is_ok());
    println!(
        "List response was: \n {:#?}",
        result.unwrap()
    );

    // TODO add this to a different function (or make a clap option to create and delete)

    // now delete the bucket
    delete_demo_bucket(&demo.s3, &bucket_name);
    demo.bucket_deleted = true;
}

fn main() {
    println!("Running tool");
    s3_demo_creation_deletion();
    println!("All done!");
}
