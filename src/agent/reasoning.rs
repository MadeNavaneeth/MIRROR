use crate::providers::{ChatMessage, ChatRequest, Provider};
use std::collections::HashMap;

/// Apply Chain-of-Thought prompt wrapper
pub fn apply_cot_prompt(user_prompt: &str) -> String {
    format!(
        "Let's approach this step-by-step:\n\n{}\n\nPlease think through this carefully and show your reasoning.",
        user_prompt
    )
}

/// Generate multiple reasoning paths with temperature sampling
pub async fn generate_reasoning_paths(
    provider: &dyn Provider,
    messages: &[ChatMessage],
    model: &str,
    num_samples: usize,
    temperature: f64,
) -> anyhow::Result<Vec<String>> {
    let mut paths = Vec::new();

    for i in 0..num_samples {
        tracing::debug!("Generating reasoning path {}/{}", i + 1, num_samples);

        let request = ChatRequest {
            messages,
            tools: None,
        };

        let response = provider.chat(request, model, temperature).await?;

        paths.push(response.text.unwrap_or_default());
    }

    Ok(paths)
}

/// Extract final answer from reasoning path
fn extract_answer(reasoning: &str) -> String {
    // Look for common answer patterns
    let patterns = [
        "Therefore,",
        "In conclusion,",
        "The answer is",
        "Final answer:",
        "To summarize,",
    ];

    for pattern in &patterns {
        if let Some(idx) = reasoning.rfind(pattern) {
            let answer = &reasoning[idx..];
            // Take the sentence containing the pattern
            if let Some(end) = answer.find('\n') {
                return answer[..end].trim().to_string();
            }
            return answer.trim().to_string();
        }
    }

    // Fallback: return last paragraph
    reasoning
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .last()
        .unwrap_or(reasoning)
        .trim()
        .to_string()
}

/// Vote on most consistent answer using majority voting
pub fn self_consistency_vote(paths: Vec<String>) -> (String, f32) {
    let mut answer_counts: HashMap<String, usize> = HashMap::new();
    let total = paths.len();

    // Extract and count answers
    for path in &paths {
        let answer = extract_answer(path);
        *answer_counts.entry(answer).or_insert(0) += 1;
    }

    // Find most common answer
    let (best_answer, count) = answer_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .unwrap_or_else(|| (paths[0].clone(), 1));

    let confidence = count as f32 / total as f32;

    (best_answer, confidence)
}

/// Main reasoning function - applies CoT and/or Self-Consistency
pub async fn reason_with_messages(
    provider: &dyn Provider,
    messages: &[ChatMessage],
    model: &str,
    config: &crate::config::ReasoningConfig,
) -> anyhow::Result<String> {
    // Apply CoT to the last user message if enabled
    let mut enhanced_messages = messages.to_vec();
    if config.cot_enabled {
        if let Some(last_msg) = enhanced_messages.last_mut() {
            if last_msg.role == "user" {
                last_msg.content = apply_cot_prompt(&last_msg.content);
            }
        }
    }

    // Use self-consistency if enabled
    if config.self_consistency_enabled {
        tracing::info!(
            "🧠 Using self-consistency with {} samples",
            config.num_samples
        );

        let paths = generate_reasoning_paths(
            provider,
            &enhanced_messages,
            model,
            config.num_samples,
            config.sampling_temperature,
        )
        .await?;

        let (answer, confidence) = self_consistency_vote(paths);

        tracing::info!("✅ Consensus confidence: {:.1}%", confidence * 100.0);

        if confidence < config.consensus_threshold {
            tracing::warn!(
                "⚠️  Low consensus ({:.1}%), consider increasing num_samples",
                confidence * 100.0
            );
        }

        Ok(answer)
    } else {
        // Standard single-path reasoning
        let request = ChatRequest {
            messages: &enhanced_messages,
            tools: None,
        };

        let response = provider
            .chat(request, model, config.sampling_temperature)
            .await?;

        Ok(response.text_or_empty().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_cot_prompt() {
        let prompt = "What is 2+2?";
        let cot_prompt = apply_cot_prompt(prompt);
        assert!(cot_prompt.contains("step-by-step"));
        assert!(cot_prompt.contains(prompt));
    }

    #[test]
    fn test_extract_answer() {
        let reasoning =
            "First, we analyze the problem.\nThen we solve it.\nTherefore, the answer is 42.";
        let answer = extract_answer(reasoning);
        assert!(answer.contains("Therefore"));
        assert!(answer.contains("42"));
    }

    #[test]
    fn test_self_consistency_vote() {
        let paths = vec![
            "Therefore, use tabs".to_string(),
            "Therefore, use tabs".to_string(),
            "The answer is drawer navigation".to_string(),
        ];

        let (answer, confidence) = self_consistency_vote(paths);
        assert!(answer.contains("tabs"));
        assert!((confidence - 0.666).abs() < 0.01);
    }
}
