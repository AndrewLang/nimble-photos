# Image process pipeline

Process uploaded images

## Pre Steps

_None_

## Main Steps

### 1. Create a new branch

Create a new git branch, name image-processing

### 2. Define Image categorize policy trait

Define a trait ImageCategorizer in backend that categorize images by certain criteria. E.g. by date taken, by hash value, etc. 

No implementation.

### 3. Define ImageCategorizer

Implement ImageCategorizerRegistry for Imagecategorizer.
1. Imagecategorizer instance can be get by name
2. Only create the instance when user request an Imagecategorizer
3. Imagecategorizer has fn categorize(source_file, desitnation), destination is a folder

### 4. Implement HashImagecategorizer

HashImagecategorizer use file hash value to categorize files.

1. Use HashService to get hash value of give file
2. HashImagecategorizer moves files from source to destination folder in this way.
   * get file hash value, e.g 1a2b3c4d5f
   * move file from source to destionation/1a/2b/file_name.jpg
3. Do it the best performance way.

### 5. Implement DateImagecategorizer

DateImagecategorizer use file date taken value to categorize image files.

1. For image file, date taken from exif metadata, if no exif date found, use file create date instead
2. DateImagecategorizer moves files from source to destination folder in this way.
   * get file date value, use a format for date, e.g. 2026-10-25
   * move file from source to destionation/2026-10-25/file_name.jpg
3. Do it the best performance way.

### 6. Implement image process pipeline

Image process pipeline is composed with image process steps, there is context share between the steps.

1. Implement ImageProcessContext, context is initialized before run pipeline
2. Define a trait ImageProcessStep
3. Implement Step extract exif metadata
4. Implement step compute hash of the file
5. Implement step extract thumnbnail, thumbnail file location come from the context.
6. Implement step get preview of the image
7. Categorize images by configured categorizer
8. Save image info and metadata to database.
9. Pipeline should run in background.

### 7. Process uploaded images

UploadPhotosHandler handles images uploading, afte file uploaded, start process the image with the pipeline.

## Post Steps

### 1. Unit test

When producing Rust code, also generate corresponding unit tests for the covered functionality. Place the test files under the `tests/` directory and ensure the tests pass.

### 2. Commit

Commit changes with meaningful message, do not push the commit.
