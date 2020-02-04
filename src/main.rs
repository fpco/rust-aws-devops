extern crate rusoto_core;
extern crate rusoto_s3;

use std::env;
use std::str;
use std::fs::File;
use std::io::Read;

use rusoto_core::{Region, RusotoError};
use rusoto_s3::{
    S3, S3Client,
    CreateBucketRequest, DeleteBucketRequest,
    ListObjectsV2Request,
    PutObjectRequest, DeleteObjectRequest
};
use structopt::StructOpt;

struct S3Demo {
    s3: S3Client,
    bucket_name: String,
}

impl S3Demo {
    // construct a new S3 demo struct
    fn new(bucket_name: String) -> S3Demo {
        let region = if let Ok(endpoint) = env::var("S3_ENDPOINT") {
            let region = Region::Custom {
                name: "us-east-1".to_string(),
                endpoint,
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
            bucket_name,
        }
    }

    fn put_object(&self, dest_filename: &str, local_filename: &str) {
        let mut contents: Vec<u8> = Vec::new();
        let mut f = File::open(local_filename).unwrap();

        match f.read_to_end(&mut contents) {
            Err(why) => panic!("Error opening file to send to S3: {}", why),
            Ok(_) => {
                let req = PutObjectRequest {
                    bucket: self.bucket_name.clone(),
                    key: dest_filename.to_string(),
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
            bucket: self.bucket_name.clone(),
            key,
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
        bucket: demo.bucket_name.clone(),
    };

    let result = demo.s3.delete_bucket(delete_bucket_req).sync();
    println!("{:#?}", result);
    if let Err(e) = result { match e {
        RusotoError::Unknown(ref e) => panic!(
            "Couldn't delete bucket because: {}",
            str::from_utf8(&e.body).unwrap()
        ),
        err => panic!("Error from S3 different than expected, was: {}", err),
    }};
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
                bucket: demo.bucket_name.clone(),
                key: dest_filename.to_string(),
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
        bucket: demo.bucket_name.clone(),
        start_after: Some("foo".to_string()),
        ..Default::default()
    };
    let result = demo.s3.list_objects_v2(list_obj_req).sync();
    assert!(result.is_ok());
    println!(
        "List response was: \n {:#?}",
        result.unwrap()
    );
}

#[derive(Debug, StructOpt)]
#[structopt(name = "RustAWSDevops",
            version = "0.1.0",
            author = "Mike McGirr <mike@fpcomplete.com>")]
enum Opt {
    #[structopt(about = "Add the specified file to the bucket")]
    AddObject {
        #[structopt(help = "bucket name")]
        bucket: String,
        #[structopt(help = "file name")]
        file: String,
    },
    #[structopt(about = "Create a new bucket with the given name")]
    Create {
        #[structopt(help = "bucket name")]
        bucket: String,
    },
    #[structopt(about = "Try to delete the bucket with the given name")]
    Delete {
        #[structopt(help = "bucket name")]
        bucket: String,
    },
    #[structopt(about = "Try to find the bucket with the given name and list its objects")]
    List {
        #[structopt(help = "bucket name")]
        bucket: String,
    },
    #[structopt(about = "Remove the specified object from the bucket")]
    DeleteObject {
        #[structopt(help = "bucket name")]
        bucket: String,
        #[structopt(help = "file name")]
        file: String,
    },

}

fn main() {
    println!("Running tool");
    let opt = Opt::from_args();
    // println!("{:?}", opt);

    match opt {
        Opt::Create { bucket: bucket_name } => {
            println!("Attempting to create a bucket called: {}", bucket_name);
            let demo = S3Demo::new(bucket_name);
            create_demo_bucket(&demo);
        },
        Opt::Delete { bucket: bucket_name } => {
            println!("Attempting to delete the bucket named: {}", bucket_name);
            let demo = S3Demo::new(bucket_name);
            delete_demo_bucket(&demo);
        },
        Opt::List { bucket: bucket_name } => {
            println!("Attempting to find and list out the objects in the bucket called: {}", bucket_name);
            let demo = S3Demo::new(bucket_name);
            find_demo_bucket_list_objects(&demo);
        },
        Opt::AddObject { bucket: bucket_name, file: filename } => {
            println!("Attempting to the object to the bucket called: {}", bucket_name);
            let demo = S3Demo::new(bucket_name);
            demo.put_object(&filename, &filename); // the trait implementation way
            // _test_put_object(&demo, &filename, &filename); // the stand-alone function way
        },
        Opt::DeleteObject { bucket: bucket_name, file: filename } => {
            println!("Attempting to find and the object in the bucket called: {}", bucket_name);
            let demo = S3Demo::new(bucket_name);
            demo.delete_object(filename); 
        },

    }

    println!("All done!");
}
