use std::thread;
use std::time::Duration;

use tegmine_common::StartWorkflowRequest;
use tegmine_core::{ExecutionService, WorkflowService};

#[test]
fn start_workflow() {
    let start_workflow_request = r#"
    {
        "name": "my_adhoc_unregistered_workflow",  
        "workflowDef": {
            "ownerApp": "my_owner_app",
            "ownerEmail": "my_owner_email@test.com",
            "createdBy": "my_username",
            "name": "my_adhoc_unregistered_workflow",
            "description": "Test Workflow setup",
            "version": 1,
            "tasks": [
                {
                    "name": "fetch_data",
                    "type": "HTTP",
                    "taskReferenceName": "fetch_data",
                    "inputParameters": {
                        "http_request": {
                            "connectionTimeOut": "3600",
                            "readTimeOut": "3600",
                            "uri": "${workflow.input.uri}",
                            "method": "GET",
                            "accept": "application/json",
                            "content-Type": "application/json",
                            "headers": { }
                        }
                    },
                    "taskDefinition": {
                        "name": "fetch_data",
                        "retryCount": 0,
                        "timeoutSeconds": 3600,
                        "timeoutPolicy": "TIME_OUT_WF",
                        "retryLogic": "FIXED",
                        "retryDelaySeconds": 0,
                        "responseTimeoutSeconds": 3000
                    }
                }
            ],
            "outputParameters": {
            }
        },
        "input": {  
            "param1": "value1",
            "param2": "value2"
        }
    }"#;
    let start_workflow_request: serde_json::Value =
        serde_json::from_str(start_workflow_request).expect("parse json failed");
    let start_workflow_request: StartWorkflowRequest = start_workflow_request
        .try_into()
        .expect("parse StartWorkflowRequest failed");
    let workflow_instance_id =
        WorkflowService::start_workflow(start_workflow_request).expect("start_workflow failed");
    eprintln!("workflow_instance_id is: {}", workflow_instance_id);

    let mut workflow = ExecutionService::get_execution_status(workflow_instance_id.as_str(), false)
        .expect("get_execution_status failed");
    while !workflow.status.is_terminal() {
        thread::sleep(Duration::from_millis(100));
        workflow = ExecutionService::get_execution_status(workflow_instance_id.as_str(), false)
            .expect("get_execution_status failed");
    }

    if workflow.status.is_successful() {
        eprintln!("workflow execute successful");
        eprintln!("output: {:?}", workflow.workflow.expect("not none").output);
    } else {
        assert!(false, "workflow execute failed");
    }
}

#[test]
fn start_workflow_registered() {
    let start_workflow_request = r#"
    {
        "name": "myWorkflow",  
        "version": 1,  
        "correlationId": "corr1",  
        "priority": 1,  
        "input": {  
            "param1": "value1",
            "param2": "value2"
        },
        "taskToDomain": {}
    }"#;
    let start_workflow_request: serde_json::Value =
        serde_json::from_str(start_workflow_request).expect("parse json failed");
    let start_workflow_request: StartWorkflowRequest = start_workflow_request
        .try_into()
        .expect("parse StartWorkflowRequest failed");
    let workflow_instance_id =
        WorkflowService::start_workflow(start_workflow_request).expect("start_workflow failed");
    eprintln!("workflow_instance_id is: {}", workflow_instance_id);
}
