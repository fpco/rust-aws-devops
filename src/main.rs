extern crate rusoto_core;
extern crate rusoto_s3;
// extern crate time;
extern crate clap;

use std::env;
use std::str;
// use time::Time;
use clap::{App, Arg};

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

fn create_demo_bucket(demo: &S3Demo, bucket_name: &String) {
    
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

// TODO a function to add an item
// TODO and break this and listing the items into a different function
// that we then call here like we eventually do with deletion.
// Or rig these up with CLI options with CLAP

// test listing buckets and items in bucket
fn find_demo_bucket_list_objects(s3: &S3Client, bucket_name: &String) {

    // let bucket_name = format!("rust-devops-webinar-demo-bucket-{}", Time::now().second());
    // let mut demo = S3Demo::new(bucket_name.clone());

    // now lets check for our bucket and list items in the one we created
    let resp = s3.list_buckets().sync();
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
    let result = s3.list_objects_v2(list_obj_req).sync();
    assert!(result.is_ok());
    println!(
        "List response was: \n {:#?}",
        result.unwrap()
    );
}

fn main() {
    println!("Running tool");

    let matches = App::new("RustAWSDevops")
        .version("0.1.0")
        .author("Mike McGirr <mike@fpcomplete.com>")
        .about("A very small devops tool written in Rust")
        .subcommand(
            App::new("create")
                .about("Create a new bucket with the given name")
                .arg( // TODO test
                    Arg::with_name("bucket")
                        .help("bucket name")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("delete")
                .about("Try to delete the bucket with the given name")
                .arg( // TODO test
                    Arg::with_name("bucket")
                        .help("bucket name")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("list")
                .about("Try to find the bucket with the given name and list its objects")
                .arg( // TODO test
                    Arg::with_name("bucket")
                        .help("bucket name")
                        .index(1)
                        .required(true),
                ),
        )
        .get_matches();

    // check what action the operator wants to do
    if let Some(ref matches) = matches.subcommand_matches("create") {
        let bucket_name = matches.value_of("bucket").unwrap().to_string();
        println!("Attempting to create a bucket called: {}", bucket_name.clone());
        let demo = S3Demo::new(bucket_name.clone());
        create_demo_bucket(&demo, &bucket_name);
    }

    if let Some(ref matches) = matches.subcommand_matches("delete") {
        let bucket_name = matches.value_of("bucket").unwrap().to_string();
        println!("Attempting to delete the bucket named: {}", bucket_name.clone());
        let demo = S3Demo::new(bucket_name.clone());
        delete_demo_bucket(&demo.s3, &bucket_name);
        // demo.bucket_deleted = true;
    }

    if let Some(ref matches) = matches.subcommand_matches("list") {
        let bucket_name = matches.value_of("bucket").unwrap().to_string();
        println!("Attempting to find and list out the objects in the bucket called: {}", bucket_name.clone());
        let demo = S3Demo::new(bucket_name.clone());
        find_demo_bucket_list_objects(&demo.s3, &bucket_name);
    }

    // Handle if no arg is given to the tool

    // TODO look into switching the above to this style of matching
    // match matches.subcommand_name() {
    //     Some("create") => create_demo_bucket(client: &S3Demo, bucket_name: String),
    //     None => println!("No command given. It's unclear what action to take."),
    //     _ => println!("Unknown subcommand. No action taken")
    // }

    println!("All done!");
}
