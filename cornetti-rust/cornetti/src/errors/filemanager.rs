#[cfg(feature = "filemanager")]
filemanager_errors(500, log_level: Error): {
    *file_operation_error   => "File operation error",
    *image_processing_error => "Image processing error",
},
