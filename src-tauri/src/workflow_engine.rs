// Workflow Engine — DAG-based execution engine

use crate::database::workflow_runs;
use crate::storage::workflows::{
    WorkflowConfig, WorkflowEdge, WorkflowNode, WorkflowNodeType,
};
use crate::AppState;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

// ── Graph Utilities ─────────────────────────────────────

/// Build adjacency list: source_id → [edges]
fn build_adjacency_list(edges: &[WorkflowEdge]) -> HashMap<String, Vec<WorkflowEdge>> {
    let mut adj: HashMap<String, Vec<WorkflowEdge>> = HashMap::new();
    for edge in edges {
        adj.entry(edge.source.clone())
            .or_default()
            .push(edge.clone());
    }
    adj
}

/// Build predecessor map: target_id → Set<source_id>
fn build_predecessor_map(
    nodes: &[WorkflowNode],
    edges: &[WorkflowEdge],
) -> HashMap<String, HashSet<String>> {
    let mut preds: HashMap<String, HashSet<String>> = HashMap::new();
    for node in nodes {
        preds.insert(node.id.clone(), HashSet::new());
    }
    for edge in edges {
        preds
            .entry(edge.target.clone())
            .or_default()
            .insert(edge.source.clone());
    }
    preds
}

/// Topological sort using Kahn's algorithm, returning layers for parallel execution
fn topological_layers(
    nodes: &[WorkflowNode],
    edges: &[WorkflowEdge],
) -> Result<Vec<Vec<String>>, String> {
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();

    for node in nodes {
        in_degree.entry(node.id.clone()).or_insert(0);
        adj.entry(node.id.clone()).or_default();
    }

    for edge in edges {
        *in_degree.entry(edge.target.clone()).or_insert(0) += 1;
        adj.entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
    }

    let mut queue: VecDeque<String> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(id, _)| id.clone())
        .collect();

    let mut layers: Vec<Vec<String>> = Vec::new();
    let mut visited = 0;

    while !queue.is_empty() {
        let layer: Vec<String> = queue.drain(..).collect();
        visited += layer.len();

        let mut next_queue = VecDeque::new();
        for node_id in &layer {
            if let Some(successors) = adj.get(node_id) {
                for succ in successors {
                    if let Some(deg) = in_degree.get_mut(succ) {
                        *deg -= 1;
                        if *deg == 0 {
                            next_queue.push_back(succ.clone());
                        }
                    }
                }
            }
        }

        layers.push(layer);
        queue = next_queue;
    }

    if visited != nodes.len() {
        return Err("Workflow graph contains a cycle".to_string());
    }

    Ok(layers)
}

/// Validate the workflow graph
fn validate_graph(nodes: &[WorkflowNode], edges: &[WorkflowEdge]) -> Result<(), String> {
    // Exactly 1 input node
    let input_count = nodes
        .iter()
        .filter(|n| matches!(n.node_type, WorkflowNodeType::Input))
        .count();
    if input_count != 1 {
        return Err(format!(
            "Workflow must have exactly 1 input node, found {}",
            input_count
        ));
    }

    // At least 1 output node
    let output_count = nodes
        .iter()
        .filter(|n| matches!(n.node_type, WorkflowNodeType::Output))
        .count();
    if output_count == 0 {
        return Err("Workflow must have at least 1 output node".to_string());
    }

    // All agent nodes must have agentId
    for node in nodes {
        if matches!(node.node_type, WorkflowNodeType::Agent) {
            if node.data.agent_id.as_ref().map_or(true, |id| id.is_empty()) {
                return Err(format!(
                    "Agent node '{}' must have an agentId",
                    node.data.label
                ));
            }
        }
    }

    // No cycles (Kahn's algorithm will detect this)
    topological_layers(nodes, edges)?;

    Ok(())
}

// ── Engine ──────────────────────────────────────────────

pub struct WorkflowEngine {
    active_runs: HashMap<String, Arc<AtomicBool>>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            active_runs: HashMap::new(),
        }
    }

    /// Execute a workflow
    pub async fn execute(
        &mut self,
        app: &AppHandle,
        state: &AppState,
        workflow: &WorkflowConfig,
        input: &str,
        trigger_type: &str,
    ) -> Result<Value, String> {
        // Validate
        validate_graph(&workflow.nodes, &workflow.edges)?;

        // Generate run ID
        let run_id = format!(
            "run-{}-{}",
            chrono::Utc::now().timestamp_millis(),
            &uuid::Uuid::new_v4().to_string()[..6]
        );

        // Create cancellation flag
        let cancelled = Arc::new(AtomicBool::new(false));
        self.active_runs
            .insert(run_id.clone(), cancelled.clone());

        // Create run record
        let now = chrono::Utc::now().timestamp_millis();
        state.db_manager.with_runs_conn(|conn| {
            conn.execute(
                "INSERT INTO workflow_runs (id, workflow_id, status, trigger_type, trigger_input, started_at, steps_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    &run_id,
                    &workflow.id,
                    "running",
                    trigger_type,
                    input,
                    now,
                    "[]",
                ],
            )
            .map_err(|e| format!("Failed to insert run: {}", e))?;
            Ok(())
        })?;

        // Execute
        let result = self
            .execute_inner(app, state, workflow, input, &run_id, &cancelled)
            .await;

        // Update run status
        let (status, final_output, error) = match &result {
            Ok(output) => ("completed", Some(output.clone()), None),
            Err(e) if e.contains("cancelled") => ("cancelled", None, Some(e.clone())),
            Err(e) => ("failed", None, Some(e.clone())),
        };

        let completed_at = chrono::Utc::now().timestamp_millis();
        let _ = state.db_manager.with_runs_conn(|conn| {
            conn.execute(
                "UPDATE workflow_runs SET status = ?1, completed_at = ?2, final_output = ?3, error = ?4 WHERE id = ?5",
                rusqlite::params![status, completed_at, final_output, error, &run_id],
            )
            .map_err(|e| format!("Failed to update run: {}", e))
        });

        // Clean up
        self.active_runs.remove(&run_id);

        // Get run for response
        let run = state
            .db_manager
            .with_runs_conn(|conn| workflow_runs::get_run(conn, &run_id))?;

        match result {
            Ok(_) => Ok(json!({ "success": true, "run": run })),
            Err(e) => Ok(json!({ "success": false, "error": e, "run": run })),
        }
    }

    async fn execute_inner(
        &self,
        app: &AppHandle,
        state: &AppState,
        workflow: &WorkflowConfig,
        user_input: &str,
        run_id: &str,
        cancelled: &Arc<AtomicBool>,
    ) -> Result<String, String> {
        let layers = topological_layers(&workflow.nodes, &workflow.edges)?;
        let adj = build_adjacency_list(&workflow.edges);
        let preds = build_predecessor_map(&workflow.nodes, &workflow.edges);
        let node_map: HashMap<&str, &WorkflowNode> =
            workflow.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        let mut node_outputs: HashMap<String, String> = HashMap::new();
        let mut condition_routes: HashMap<String, bool> = HashMap::new();

        for layer in &layers {
            if cancelled.load(Ordering::Relaxed) {
                return Err(format!("Run {} was cancelled", run_id));
            }

            for node_id in layer {
                let node = node_map
                    .get(node_id.as_str())
                    .ok_or(format!("Node not found: {}", node_id))?;

                // Check if this node is reachable (not blocked by condition routing)
                if is_node_blocked(node_id, &preds, &condition_routes) {
                    continue;
                }

                // Build node input from predecessors
                let node_input =
                    build_node_input(node_id, &preds, &node_outputs, &condition_routes);

                // Emit step progress: running
                emit_step_progress(app, run_id, node_id, "running", None);

                // Execute node
                let output = match node.node_type {
                    WorkflowNodeType::Input => user_input.to_string(),
                    WorkflowNodeType::Agent => {
                        execute_agent_node(
                            state,
                            node,
                            &node_input,
                            &workflow.id,
                            run_id,
                            cancelled,
                        )
                        .await?
                    }
                    WorkflowNodeType::Condition => {
                        execute_condition_node(
                            node,
                            &node_input,
                            &adj,
                            &mut condition_routes,
                        )
                    }
                    WorkflowNodeType::Merge => {
                        execute_merge_node(node, node_id, &preds, &node_outputs, &condition_routes)
                    }
                    WorkflowNodeType::Output => node_input.clone(),
                };

                node_outputs.insert(node_id.clone(), output.clone());
                emit_step_progress(app, run_id, node_id, "completed", Some(&output));
            }
        }

        // Collect output node values
        let output_parts: Vec<String> = workflow
            .nodes
            .iter()
            .filter(|n| matches!(n.node_type, WorkflowNodeType::Output))
            .filter_map(|n| node_outputs.get(&n.id))
            .cloned()
            .collect();

        Ok(output_parts.join("\n"))
    }

    /// Cancel a running workflow
    pub fn cancel(&mut self, run_id: &str) {
        if let Some(flag) = self.active_runs.get(run_id) {
            flag.store(true, Ordering::Relaxed);
        }
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ── Node Execution ──────────────────────────────────────

/// Check if a node is blocked by condition routing
fn is_node_blocked(
    node_id: &str,
    preds: &HashMap<String, HashSet<String>>,
    condition_routes: &HashMap<String, bool>,
) -> bool {
    if let Some(predecessors) = preds.get(node_id) {
        for pred_id in predecessors {
            let route_key = format!("{}->{}", pred_id, node_id);
            if let Some(&active) = condition_routes.get(&route_key) {
                if !active {
                    return true;
                }
            }
        }
    }
    false
}

/// Build input for a node from its predecessors
fn build_node_input(
    node_id: &str,
    preds: &HashMap<String, HashSet<String>>,
    node_outputs: &HashMap<String, String>,
    condition_routes: &HashMap<String, bool>,
) -> String {
    let mut inputs = Vec::new();

    if let Some(predecessors) = preds.get(node_id) {
        for pred_id in predecessors {
            let route_key = format!("{}->{}", pred_id, node_id);
            if let Some(&active) = condition_routes.get(&route_key) {
                if !active {
                    continue;
                }
            }
            if let Some(output) = node_outputs.get(pred_id) {
                inputs.push(output.clone());
            }
        }
    }

    if inputs.len() == 1 {
        inputs.into_iter().next().expect("must have exactly one input")
    } else if inputs.is_empty() {
        String::new()
    } else {
        inputs.join("\n")
    }
}

/// Execute an agent node via Gateway RPC
async fn execute_agent_node(
    state: &AppState,
    node: &WorkflowNode,
    input: &str,
    workflow_id: &str,
    run_id: &str,
    cancelled: &Arc<AtomicBool>,
) -> Result<String, String> {
    let agent_id = node
        .data
        .agent_id
        .as_ref()
        .ok_or("Agent node missing agentId")?;

    let session_key = format!("workflow-{}-run-{}-step-{}", workflow_id, run_id, node.id);

    // Apply prompt template
    let message = if let Some(template) = &node.data.prompt_template {
        if !template.is_empty() {
            template.replace("{{input}}", input)
        } else {
            input.to_string()
        }
    } else {
        input.to_string()
    };

    if cancelled.load(Ordering::Relaxed) {
        return Err(format!("Run {} was cancelled", run_id));
    }

    // Send chat request via Gateway RPC
    let params = json!({
        "sessionKey": session_key,
        "message": message,
        "agentId": agent_id,
        "deliver": false,
    });

    let result = state
        .gateway_client
        .rpc("chat.send", Some(params), 120000)
        .await?;

    // Extract response text
    let response = result
        .get("content")
        .or_else(|| result.get("text"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            serde_json::to_string(&result).unwrap_or_default()
        });

    Ok(response)
}

/// Execute a condition node — evaluate rules and route edges
fn execute_condition_node(
    node: &WorkflowNode,
    input: &str,
    adj: &HashMap<String, Vec<WorkflowEdge>>,
    condition_routes: &mut HashMap<String, bool>,
) -> String {
    let rules = node.data.condition_rules.as_deref().unwrap_or(&[]);
    let out_edges = adj.get(&node.id).cloned().unwrap_or_default();

    let mut matched_handle: Option<String> = None;

    // Evaluate rules in order, skipping default
    for rule in rules {
        if rule.is_default == Some(true) {
            continue;
        }

        let matched = match rule.rule_type.as_str() {
            "keyword" => input.contains(&rule.value),
            "regex" => regex::Regex::new(&rule.value)
                .map(|re| re.is_match(input))
                .unwrap_or(false),
            _ => false,
        };

        if matched {
            matched_handle = Some(rule.handle.clone());
            break;
        }
    }

    // Fallback to default rule, then first rule
    if matched_handle.is_none() {
        let default_rule = rules.iter().find(|r| r.is_default == Some(true));
        if let Some(dr) = default_rule {
            matched_handle = Some(dr.handle.clone());
        } else if !rules.is_empty() {
            matched_handle = Some(rules[0].handle.clone());
        }
    }

    // Record routing decisions
    for edge in &out_edges {
        let is_active = matched_handle.as_ref().map_or(false, |handle| {
            edge.source_handle.as_ref() == Some(handle)
        });
        condition_routes.insert(format!("{}->{}", node.id, edge.target), is_active);
    }

    input.to_string()
}

/// Execute a merge node — combine outputs from multiple branches
fn execute_merge_node(
    node: &WorkflowNode,
    node_id: &str,
    preds: &HashMap<String, HashSet<String>>,
    node_outputs: &HashMap<String, String>,
    condition_routes: &HashMap<String, bool>,
) -> String {
    let strategy = node
        .data
        .merge_strategy
        .as_deref()
        .unwrap_or("concat");

    let mut branch_outputs = Vec::new();
    if let Some(predecessors) = preds.get(node_id) {
        for pred_id in predecessors {
            let route_key = format!("{}->{}", pred_id, node_id);
            if let Some(&active) = condition_routes.get(&route_key) {
                if !active {
                    continue;
                }
            }
            if let Some(val) = node_outputs.get(pred_id) {
                branch_outputs.push(val.clone());
            }
        }
    }

    match strategy {
        "concat" => branch_outputs.join("\n---\n"),
        "first" => branch_outputs.into_iter().next().unwrap_or_default(),
        "custom" => {
            let mut template = node
                .data
                .merge_template
                .clone()
                .unwrap_or_default();
            for (i, output) in branch_outputs.iter().enumerate() {
                let placeholder = format!("{{{{branch_{}}}}}", i);
                template = template.replace(&placeholder, output);
            }
            template
        }
        _ => branch_outputs.join("\n---\n"),
    }
}

/// Emit step progress event
fn emit_step_progress(
    app: &AppHandle,
    run_id: &str,
    node_id: &str,
    status: &str,
    output: Option<&str>,
) {
    let _ = app.emit(
        "workflow_stepProgress",
        json!({
            "runId": run_id,
            "nodeId": node_id,
            "status": status,
            "output": output,
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::workflows::{ConditionRule, NodePosition, WorkflowNodeData};

    fn make_node(id: &str, node_type: WorkflowNodeType, label: &str) -> WorkflowNode {
        let is_agent = matches!(node_type, WorkflowNodeType::Agent);
        WorkflowNode {
            id: id.to_string(),
            node_type,
            position: NodePosition { x: 0.0, y: 0.0 },
            data: WorkflowNodeData {
                label: label.to_string(),
                agent_id: if is_agent {
                    Some("agent-1".to_string())
                } else {
                    None
                },
                prompt_template: None,
                condition_type: None,
                condition_rules: None,
                merge_strategy: None,
                merge_template: None,
            },
        }
    }

    fn make_edge(id: &str, source: &str, target: &str) -> WorkflowEdge {
        WorkflowEdge {
            id: id.to_string(),
            source: source.to_string(),
            target: target.to_string(),
            source_handle: None,
            target_handle: None,
            label: None,
        }
    }

    #[test]
    fn test_validate_graph_valid() {
        let nodes = vec![
            make_node("n1", WorkflowNodeType::Input, "Input"),
            make_node("n2", WorkflowNodeType::Agent, "Agent"),
            make_node("n3", WorkflowNodeType::Output, "Output"),
        ];
        let edges = vec![
            make_edge("e1", "n1", "n2"),
            make_edge("e2", "n2", "n3"),
        ];

        assert!(validate_graph(&nodes, &edges).is_ok());
    }

    #[test]
    fn test_validate_graph_no_input() {
        let nodes = vec![
            make_node("n1", WorkflowNodeType::Agent, "Agent"),
            make_node("n2", WorkflowNodeType::Output, "Output"),
        ];
        let edges = vec![make_edge("e1", "n1", "n2")];

        assert!(validate_graph(&nodes, &edges).is_err());
    }

    #[test]
    fn test_validate_graph_no_output() {
        let nodes = vec![
            make_node("n1", WorkflowNodeType::Input, "Input"),
            make_node("n2", WorkflowNodeType::Agent, "Agent"),
        ];
        let edges = vec![make_edge("e1", "n1", "n2")];

        assert!(validate_graph(&nodes, &edges).is_err());
    }

    #[test]
    fn test_validate_graph_cycle() {
        let nodes = vec![
            make_node("n1", WorkflowNodeType::Input, "Input"),
            make_node("n2", WorkflowNodeType::Agent, "Agent"),
            make_node("n3", WorkflowNodeType::Output, "Output"),
        ];
        let edges = vec![
            make_edge("e1", "n1", "n2"),
            make_edge("e2", "n2", "n3"),
            make_edge("e3", "n3", "n2"), // cycle
        ];

        assert!(validate_graph(&nodes, &edges).is_err());
    }

    #[test]
    fn test_topological_layers() {
        let nodes = vec![
            make_node("n1", WorkflowNodeType::Input, "Input"),
            make_node("n2", WorkflowNodeType::Agent, "A"),
            make_node("n3", WorkflowNodeType::Agent, "B"),
            make_node("n4", WorkflowNodeType::Output, "Output"),
        ];
        let edges = vec![
            make_edge("e1", "n1", "n2"),
            make_edge("e2", "n1", "n3"),
            make_edge("e3", "n2", "n4"),
            make_edge("e4", "n3", "n4"),
        ];

        let layers = topological_layers(&nodes, &edges).unwrap();
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0], vec!["n1"]);
        assert!(layers[1].contains(&"n2".to_string()));
        assert!(layers[1].contains(&"n3".to_string()));
        assert_eq!(layers[2], vec!["n4"]);
    }

    #[test]
    fn test_condition_routing() {
        let mut condition_routes = HashMap::new();

        let node = WorkflowNode {
            id: "cond1".to_string(),
            node_type: WorkflowNodeType::Condition,
            position: NodePosition { x: 0.0, y: 0.0 },
            data: WorkflowNodeData {
                label: "Condition".to_string(),
                agent_id: None,
                prompt_template: None,
                condition_type: None,
                condition_rules: Some(vec![
                    ConditionRule {
                        id: "r1".to_string(),
                        handle: "yes".to_string(),
                        rule_type: "keyword".to_string(),
                        value: "hello".to_string(),
                        is_default: None,
                    },
                    ConditionRule {
                        id: "r2".to_string(),
                        handle: "no".to_string(),
                        rule_type: "keyword".to_string(),
                        value: "".to_string(),
                        is_default: Some(true),
                    },
                ]),
                merge_strategy: None,
                merge_template: None,
            },
        };

        let mut adj = HashMap::new();
        adj.insert(
            "cond1".to_string(),
            vec![
                WorkflowEdge {
                    id: "e1".to_string(),
                    source: "cond1".to_string(),
                    target: "n2".to_string(),
                    source_handle: Some("yes".to_string()),
                    target_handle: None,
                    label: None,
                },
                WorkflowEdge {
                    id: "e2".to_string(),
                    source: "cond1".to_string(),
                    target: "n3".to_string(),
                    source_handle: Some("no".to_string()),
                    target_handle: None,
                    label: None,
                },
            ],
        );

        execute_condition_node(&node, "hello world", &adj, &mut condition_routes);

        assert_eq!(condition_routes.get("cond1->n2"), Some(&true));
        assert_eq!(condition_routes.get("cond1->n3"), Some(&false));
    }

    #[test]
    fn test_merge_concat() {
        let node = WorkflowNode {
            id: "merge1".to_string(),
            node_type: WorkflowNodeType::Merge,
            position: NodePosition { x: 0.0, y: 0.0 },
            data: WorkflowNodeData {
                label: "Merge".to_string(),
                agent_id: None,
                prompt_template: None,
                condition_type: None,
                condition_rules: None,
                merge_strategy: Some("concat".to_string()),
                merge_template: None,
            },
        };

        let mut preds = HashMap::new();
        let mut pred_set = HashSet::new();
        pred_set.insert("n1".to_string());
        pred_set.insert("n2".to_string());
        preds.insert("merge1".to_string(), pred_set);

        let mut outputs = HashMap::new();
        outputs.insert("n1".to_string(), "Branch A".to_string());
        outputs.insert("n2".to_string(), "Branch B".to_string());

        let result = execute_merge_node(&node, "merge1", &preds, &outputs, &HashMap::new());
        assert!(result.contains("Branch A"));
        assert!(result.contains("Branch B"));
    }
}
