use tegmine_common::{TaskDef, WorkflowDef};
use tegmine_core::MetadataService;

#[test]
pub fn register_workflow() {
    let workflow_def = r#"
    {
        "name": "mail_a_box",
        "description": "shipping Workflow",
        "version": 1,
        "tasks": [
            {
                "name": "shipping_info",
                "taskReferenceName": "shipping_info_ref",
                "inputParameters": {
                    "account": "${workflow.input.accountNumber}"
                },
                "type": "SIMPLE"
            },
            {
                "name": "shipping_task",
                "taskReferenceName": "shipping_task_ref",
                "inputParameters": {
                    "name": "${shipping_info_ref.output.name}",
                    "streetAddress": "${shipping_info_ref.output.streetAddress}",
                    "city": "${shipping_info_ref.output.city}",
                    "state": "${shipping_info_ref.output.state}",
                    "zipcode": "${shipping_info_ref.output.zipcode}"
                },
                "type": "SIMPLE"
            }
        ],
        "outputParameters": {
            "trackingNumber": "${shipping_task_ref.output.trackingNumber}"
        },
        "failureWorkflow": "shipping_issues",
        "restartable": true,
        "workflowStatusListenerEnabled": true,
        "ownerEmail": "example@example.com",
        "timeoutPolicy": "ALERT_ONLY",
        "timeoutSeconds": 0,
        "variables": {},
        "inputTemplate": {}
    }"#;
    let workflow_def: serde_json::Value =
        serde_json::from_str(workflow_def).expect("parse json failed");
    let workflow_def = WorkflowDef::try_from(&workflow_def).expect("parse WorkflowDef failed");
    MetadataService::register_workflow_def(workflow_def).expect("register_workflow_def failed");
}

#[test]
fn register_task() {
    let task_def = r#"
    {
        "name": "encode_task",
        "retryCount": 3,
        "timeoutSeconds": 1200,
        "inputKeys": [
            "sourceRequestId",
            "qcElementType"
        ],
        "outputKeys": [
            "state",
            "skipped",
            "result"
        ],
        "timeoutPolicy": "TIME_OUT_WF",
        "retryLogic": "FIXED",
        "retryDelaySeconds": 600,
        "responseTimeoutSeconds": 3600,
        "pollTimeoutSeconds": 3600,
        "concurrentExecLimit": 100,
        "rateLimitFrequencyInSeconds": 60,
        "rateLimitPerFrequency": 50,
        "ownerEmail": "foo@bar.com",
        "description": "Sample Encoding task"
    }"#;
    let task_def: serde_json::Value = serde_json::from_str(task_def).expect("parse json failed");
    let task_def = TaskDef::try_from(&task_def).expect("parse TaskDef failed");
    MetadataService::register_task_def(vec![task_def], "test").expect("register_task_def failed");
}
