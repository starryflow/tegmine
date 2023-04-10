use tegmine_common::StartWorkflowRequest;
use tegmine_core::{ExecutionService, WorkflowService};

#[test]
fn start_workflow() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .is_test(true)
        .try_init();

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
                    "name": "switch_by_param",
                    "taskReferenceName": "switch_by_param",
                    "type": "SWITCH",
                    "evaluatorType": "value-param",
                    "expression": "switchCaseValue",
                    "inputParameters": {
                        "switchCaseValue": "${workflow.input.service}"
                    },
                    "decisionCases": {
                        "fedex": [
                            {
                                "name": "Set_Name_fedex",
                                "taskReferenceName": "Set_Name_fedex",
                                "type": "SET_VARIABLE",
                                "inputParameters": {
                                    "name": "Foo"
                                }
                            }
                        ],
                        "ups": [
                            {
                                "name": "Set_Name_ups",
                                "taskReferenceName": "Set_Name_ups",
                                "type": "SET_VARIABLE",
                                "inputParameters": {
                                    "name": "Bar"
                                }
                            }
                        ]
                    },
                    "defaultCase": [
                        {
                            "name": "Set_Name_default",
                            "taskReferenceName": "Set_Name_default",
                            "type": "SET_VARIABLE",
                            "inputParameters": {
                                "name": "Default"
                            }
                        }
                    ]
                }
            ],
            "outputParameters": {
                "output": "${workflow.variables.name}"
            }
        },
        "input": {  
            "service": "ups"
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

    tegmine_core::evaluate_once().expect("evaluation failed");

    let workflow = ExecutionService::get_execution_status(workflow_instance_id.as_str(), false)
        .expect("get_execution_status failed");

    if workflow.status.is_terminal() && workflow.status.is_successful() {
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
