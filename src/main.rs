extern crate rusoto_core;
extern crate rusoto_s3;
extern crate clap;

use std::env;
use std::str;
use std::fs::File;
use std::io::Read;

use clap::{App, Arg};

use rusoto_core::{Region, RusotoError};
use rusoto_s3::{
    S3, S3Client,
    CreateBucketRequest, DeleteBucketRequest,
    ListObjectsV2Request,
    PutObjectRequest, DeleteObjectRequest
};

struct S3Demo {
    s3: S3Client,
    bucket_name: String,
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
            s3: S3Client::new(region),
            bucket_name: bucket_name.to_owned(),
        }
    }

    fn put_object(&self, dest_filename: &str, local_filename: &str) {
        let mut contents: Vec<u8> = Vec::new();
        let mut f = File::open(local_filename).unwrap();

        match f.read_to_end(&mut contents) {
            Err(why) => panic!("Error opening file to send to S3: {}", why),
            Ok(_) => {
                let req = PutObjectRequest {
                    bucket: self.bucket_name.to_owned(),
                    key: dest_filename.to_owned(),
                    body: Some(contents.into()),
                    ..Default::default()
                };

                self.s3
                    .put_object(req)
                    .sync()
                    .expect("Failed to put test object");
            }
        }

    }

    fn delete_object(&self, key: String) {
        let delete_object_req = DeleteObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: key.to_owned(),
            ..Default::default()
        };

        self.s3
            .delete_object(delete_object_req)
            .sync()
            .expect("Couldn't delete object");
    }
}

fn create_demo_bucket(demo: &S3Demo) {

    let create_bucket_req = CreateBucketRequest {
        bucket: demo.bucket_name.clone(),
        ..Default::default()
    };

    // first create a bucket
    let create_bucket_resp = demo.s3.create_bucket(create_bucket_req).sync();
    assert!(create_bucket_resp.is_ok());
    println!(
        "Bucket {} created\n response:\n {:#?}",
        demo.bucket_name.clone(),
        create_bucket_resp.unwrap()
    );
}

fn delete_demo_bucket(demo: &S3Demo) {
    let delete_bucket_req = DeleteBucketRequest {
        bucket: demo.bucket_name.to_owned(),
        ..Default::default()
    };

    let result = demo.s3.delete_bucket(delete_bucket_req).sync();
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

// A function to demo another way to add an item (if you didn't want to use the trait implementation)
// This method is stand-alone function
fn _test_put_object(
    demo: &S3Demo,
    dest_filename: &str,
    local_filename: &str,
) {
    let mut f = File::open(local_filename).unwrap();
    let mut contents: Vec<u8> = Vec::new();
    match f.read_to_end(&mut contents) {
        Err(why) => panic!("Error opening file to send to S3: {}", why),
        Ok(_) => {
            let req = PutObjectRequest {
                bucket: demo.bucket_name.to_owned(),
                key: dest_filename.to_owned(),
                body: Some(contents.into()),
                ..Default::default()
            };
            let result = demo.s3.put_object(req).sync().expect("Couldn't PUT object");
            println!("{:#?}", result);
        }
    }
}

// test listing buckets and items in bucket
fn find_demo_bucket_list_objects(demo: &S3Demo) {

    // now lets check for our bucket and list items in the one we created
    let resp = demo.s3.list_buckets().sync();
    assert!(resp.is_ok());

    let resp = resp.unwrap();
    let mut bucket_found = false;
    for bucket in resp.buckets.unwrap().iter() {
        if bucket.name == Some(demo.bucket_name.clone()) {
            bucket_found = true;
            break;
        }
    }
    assert!(bucket_found);

    let list_obj_req = ListObjectsV2Request {
        bucket: demo.bucket_name.to_owned(),
        start_after: Some("foo".to_owned()),
        ..Default::default()
    };
    let result = demo.s3.list_objects_v2(list_obj_req).sync();
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
                .arg(
                    Arg::with_name("bucket")
                        .help("bucket name")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("delete")
                .about("Try to delete the bucket with the given name")
                .arg(
                    Arg::with_name("bucket")
                        .help("bucket name")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("list")
                .about("Try to find the bucket with the given name and list its objects")
                .arg(
                    Arg::with_name("bucket")
                        .help("bucket name")
                        .index(1)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("add-object")
                .about("Add the specified file to the bucket")
                .arg(
                    Arg::with_name("bucket")
                        .help("bucket name")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::with_name("file")
                        .help("file name")
                        .index(2)
                        .required(true),
                ),
        )
        .subcommand(
            App::new("delete-object")
                .about("Remove the specified object from the bucket")
                .arg(
                    Arg::with_name("bucket")
                        .help("bucket name")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::with_name("file")
                        .help("file name")
                        .index(2)
                        .required(true),
                ),
        )
        .get_matches();

    // check what action the operator wants to do
    if let Some(ref matches) = matches.subcommand_matches("create") {
        let bucket_name = matches.value_of("bucket").unwrap().to_string();
        println!("Attempting to create a bucket called: {}", bucket_name.clone());
        let demo = S3Demo::new(bucket_name.clone());
        create_demo_bucket(&demo);
    }

    if let Some(ref matches) = matches.subcommand_matches("delete") {
        let bucket_name = matches.value_of("bucket").unwrap().to_string();
        println!("Attempting to delete the bucket named: {}", bucket_name.clone());
        let demo = S3Demo::new(bucket_name.clone());
        delete_demo_bucket(&demo);
    }

    if let Some(ref matches) = matches.subcommand_matches("list") {
        let bucket_name = matches.value_of("bucket").unwrap().to_string();
        println!("Attempting to find and list out the objects in the bucket called: {}", bucket_name.clone());
        let demo = S3Demo::new(bucket_name.clone());
        find_demo_bucket_list_objects(&demo);
    }

    if let Some(ref matches) = matches.subcommand_matches("add-object") {
        let bucket_name = matches.value_of("bucket").unwrap().to_string();
        let filename = matches.value_of("file").unwrap().to_string();
        println!("Attempting to the object to the bucket called: {}", bucket_name.clone());
        let demo = S3Demo::new(bucket_name.clone());
        demo.put_object(&filename, &filename);         // the trait implementation way
        // _test_put_object(&demo, &filename, &filename); // the stand-alone function way
    }

    if let Some(ref matches) = matches.subcommand_matches("delete-object") {
        let bucket_name = matches.value_of("bucket").unwrap().to_string();
        let filename = matches.value_of("file").unwrap().to_string();
        println!("Attempting to find and the object in the bucket called: {}", bucket_name.clone());
        let demo = S3Demo::new(bucket_name.clone());
        demo.delete_object(filename); 
    }

    println!("All done!");
}
