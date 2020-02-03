## Usage

After compiling the code with `cargo build` you can use the executable produced the following ways:

Create a new bucket:

```
aws-env rust-aws-devops create fpcomplete-webinar-demo
```

Add a file (in this case `test.txt`) to the bucket (`fpcomplete-webinar-demo`):

```
aws-env rust-aws-devops add-object fpcomplete-webinar-demo test.txt
```

Search for the bucket and list the objects in it:

```
aws-env rust-aws-devops list fpcomplete-webinar-demo
```

Delete an object in the bucket:

```
aws-env rust-aws-devops delete-object fpcomplete-webinar-demo test.txt
```

Delete a bucket:

```
aws-env rust-aws-devops delete fpcomplete-webinar-demo
```

**NOTE**: These CLI instructions use `aws-env` which is a helper script to that deals with AWS creds. 
You can find it here: [aws-env](https://github.com/fpco/devops-helpers/blob/master/doc/aws/aws-env.md) 
Also note that if you want to use `aws-env` you will need to add a file called `aws-env.config` 
to the repo directory of the form:

```
region=<FILL IN>
profile=<FILL IN>
source_profile=<FILL IN (if needed)>
role_arn=arn:aws:iam::<ACCOUNT-ID>:role/<ROLE>
```
