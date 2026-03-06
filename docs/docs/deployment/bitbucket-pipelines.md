---
{
  "title": "Bitbucket Pipelines",
  "description": "Deploy your DocAnvil docs to AWS S3 with Bitbucket Pipelines"
}
---
# Deploying with Bitbucket Pipelines

This guide shows how to set up a Bitbucket Pipeline that builds your DocAnvil docs and deploys the static output to an AWS S3 bucket — every time you push to `master`.

## Prerequisites

You'll need:

- A Bitbucket repository containing your DocAnvil project
- An AWS account with an S3 bucket configured for static website hosting (or CloudFront in front of a private bucket)
- An IAM user or role with permission to write to that bucket (and optionally create CloudFront invalidations)

The minimum IAM policy for S3-only deploys:

```json
{
  "Effect": "Allow",
  "Action": ["s3:PutObject", "s3:DeleteObject", "s3:ListBucket"],
  "Resource": [
    "arn:aws:s3:::your-bucket-name",
    "arn:aws:s3:::your-bucket-name/*"
  ]
}
```

Add `cloudfront:CreateInvalidation` to the policy if you're using CloudFront.

## Repository Variables

Add the following variables in **Repository settings → Repository variables** (or under a named deployment environment):

| Variable | Required | Description |
|---|---|---|
| `AWS_ACCESS_KEY_ID` | ✅ | IAM access key ID |
| `AWS_SECRET_ACCESS_KEY` | ✅ | IAM secret access key — mark as **Secured** |
| `AWS_DEFAULT_REGION` | ✅ | AWS region, e.g. `us-east-1` |
| `S3_BUCKET` | ✅ | S3 bucket name, e.g. `docs.example.com` |
| `CLOUDFRONT_DISTRIBUTION_ID` | Optional | CloudFront distribution ID for cache invalidation |

## Pipeline Configuration

Create `bitbucket-pipelines.yml` at the root of your repository:

```yaml
image: atlassian/default-image:4

pipelines:
  branches:
    master:
      - step:
          name: Build and deploy docs
          deployment: production
          script:
            # Download the latest DocAnvil binary
            - LATEST=$(curl -sSf https://api.github.com/repos/docanvil/docanvil/releases/latest | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
            - VERSION=${LATEST#v}
            - curl -sSfL "https://github.com/docanvil/docanvil/releases/download/${LATEST}/docanvil-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz" | tar -xz -C /usr/local/bin

            # Build the docs
            - docanvil build --strict --clean --path ./docs

            # Sync the static output to S3
            - aws s3 sync docs/dist/ s3://$S3_BUCKET --delete

            # Invalidate CloudFront cache (uncomment if using CloudFront)
            # - aws cloudfront create-invalidation --distribution-id $CLOUDFRONT_DISTRIBUTION_ID --paths "/*"
```

A few things worth noting:

- **`atlassian/default-image:4`** ships with the AWS CLI pre-installed, so no extra install step is needed.
- **`deployment: production`** ties the step to your Bitbucket deployment environment. Variable values set on that environment take precedence over repo-level variables.
- **`--strict`** makes the build fail on any warnings — broken wiki-links, missing images, and other issues will stop the deploy before anything reaches S3.
- **`--delete`** in the S3 sync removes objects that no longer exist in your built output, keeping the bucket clean.

## Adjusting for Your Project Layout

The example above assumes your DocAnvil project lives in a `docs/` subdirectory of your repo. Adjust the `--path` and sync source path to match your setup:

```yaml
# DocAnvil project at the repo root
- docanvil build --strict --clean
- aws s3 sync dist/ s3://$S3_BUCKET --delete

# DocAnvil project in a subdirectory
- docanvil build --strict --clean --path ./my-docs
- aws s3 sync my-docs/dist/ s3://$S3_BUCKET --delete
```

## S3 Bucket Setup

For public static website hosting, enable **Static website hosting** in the S3 console (Bucket → Properties → Static website hosting). Set `index.html` as the index document and `404.html` as the error document — DocAnvil generates both.

For HTTPS and edge caching, put CloudFront in front of a private bucket instead:

1. Keep **Block all public access** enabled on the bucket
2. Create a CloudFront distribution with an Origin Access Control (OAC) policy
3. Point the distribution at the bucket and set the default root object to `index.html`
4. Uncomment the `create-invalidation` line in the pipeline so visitors always see fresh content after a deploy

## What Happens on Each Push

When you push to `master`:

1. Bitbucket Pipelines pulls `atlassian/default-image:4`
2. The pipeline fetches the latest DocAnvil binary from GitHub Releases — no pinned version to maintain
3. `docanvil build --strict --clean` builds a fresh static site into `docs/dist/`
4. `aws s3 sync` uploads only the changed files and removes deleted ones
5. (Optionally) CloudFront invalidates the edge cache so the new content is live within seconds

:::note
The binary download fetches the latest stable release every time. If you need reproducible builds with a pinned version, replace the dynamic version resolution with a hardcoded tag: `LATEST=v1.2.0`.
:::
