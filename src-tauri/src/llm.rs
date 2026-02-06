use serde::{Deserialize, Serialize};
use crate::models::{Segment, Word};

// Z.ai API request format (OpenAI-compatible)
#[derive(Debug, Serialize)]
struct ZAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

// Z.ai API response format
#[derive(Debug, Deserialize)]
struct ZAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

/// Clean up transcription using Z.ai GLM-4.7-Flash to fix word fragments and errors
pub async fn clean_transcript_with_llm(
    segments: &[Segment],
    api_key: &str,
) -> Result<Vec<Segment>, String> {
    eprintln!("🤖 Cleaning transcript with Z.ai GLM-4.7-flash...");

    // Build the full transcript text
    let full_text: Vec<String> = segments.iter().map(|s| s.text.clone()).collect();
    let transcript_text = full_text.join(" ");

    eprintln!("📝 Original text length: {} chars", transcript_text.len());

    // Create the prompt
    let system_prompt = "You are a transcription correction assistant. Fix word fragments and spelling errors in Spanish transcriptions while maintaining the same word count and timing.";

    let user_prompt = format!(
        r#"Fix this Spanish transcription from Whisper AI. It has word fragments that need to be joined:

Rules:
1. Join word fragments (e.g., "dis av ivo" → "dispositivo", "nego cios" → "negocios", "fr acas ar" → "fracasar")
2. Fix obvious spelling errors
3. Keep the same approximate word count (don't add or remove content)
4. Minimal punctuation
5. Return ONLY the corrected text, no explanations

Original: {}

Corrected:"#,
        transcript_text
    );

    // Call Z.ai API (OpenAI-compatible endpoint)
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let url = "https://api.z.ai/api/paas/v4/chat/completions";

    let request_body = ZAIRequest {
        model: "GLM-4.7-Flash".to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        temperature: 0.3,
    };

    eprintln!("📤 Sending request to Z.ai...");

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to call Z.ai API: {}", e))?;

    eprintln!("📥 Received response with status: {}", response.status());

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("❌ Z.ai API error {}: {}", status, error_text);
        return Err(format!("Z.ai API error {}: {}", status, error_text));
    }

    eprintln!("🔄 Parsing response...");

    let response_text = response.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    eprintln!("📄 Response preview: {}", &response_text[..response_text.len().min(200)]);

    let zai_response: ZAIResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse Z.ai response: {} - Response: {}", e, response_text))?;

    eprintln!("✅ Successfully parsed response");

    let cleaned_text = zai_response
        .choices
        .first()
        .map(|c| c.message.content.trim().to_string())
        .ok_or("No response from Z.ai")?;

    eprintln!("✨ Cleaned text length: {} chars", cleaned_text.len());
    eprintln!("📊 Sample: {}", &cleaned_text[..cleaned_text.len().min(100)]);

    // Now we need to redistribute this cleaned text back to segments with timestamps
    redistribute_text_to_segments(segments, &cleaned_text)
}

/// Redistribute cleaned text back to segments, maintaining timestamps
fn redistribute_text_to_segments(
    original_segments: &[Segment],
    cleaned_text: &str,
) -> Result<Vec<Segment>, String> {
    // Split cleaned text into words
    let cleaned_words: Vec<&str> = cleaned_text.split_whitespace().collect();

    eprintln!("🔄 Redistributing {} words to {} segments", cleaned_words.len(), original_segments.len());

    let mut result_segments = Vec::new();
    let mut word_index = 0;

    for original_segment in original_segments {
        let segment_duration = original_segment.end - original_segment.start;

        // Estimate how many words should be in this segment based on original word count
        let original_word_count = original_segment.words.len();

        // Take words for this segment (proportional to original)
        let words_for_segment: Vec<&str> = cleaned_words
            .get(word_index..word_index + original_word_count)
            .unwrap_or(&cleaned_words[word_index..])
            .to_vec();

        // Build segment text
        let segment_text = words_for_segment.join(" ");

        // Distribute timestamps evenly across words in this segment
        let mut segment_words = Vec::new();
        let time_per_word = segment_duration / words_for_segment.len() as f64;

        for (i, word_text) in words_for_segment.iter().enumerate() {
            let word_start = original_segment.start + (i as f64 * time_per_word);
            let word_end = word_start + time_per_word;

            segment_words.push(Word {
                id: format!("w{}", word_index + i),
                word: word_text.to_string(),
                start: word_start,
                end: word_end,
            });
        }

        result_segments.push(Segment {
            id: original_segment.id,
            start: original_segment.start,
            end: original_segment.end,
            text: segment_text,
            words: segment_words,
        });

        word_index += words_for_segment.len();
    }

    Ok(result_segments)
}
