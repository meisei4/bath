import os
import json
import subprocess
import logging
from typing import List, Dict, Optional

import whisper
import spacy
from transformers import M2M100ForConditionalGeneration, M2M100Tokenizer
from sacremoses import MosesDetokenizer

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(message)s',
    handlers=[
        logging.FileHandler("translation_pipeline.log"),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

OUTPUT_DIR: str = os.path.join(os.getcwd(), "output_audio")
WHISPER_SOURCE_AUDIO: str = os.path.join(OUTPUT_DIR, "whisper_source_audio.wav")
WHISPER_TRANSCRIPTION_FILE: str = os.path.join(OUTPUT_DIR, "whisper_transcription.json")
SENTENCE_SEGMENT_FILE: str = os.path.join(OUTPUT_DIR, "sentence_segments.json")
FINAL_SRT_FILE: str = os.path.join(OUTPUT_DIR, "translated_subtitles.srt")
WHISPER_YOUTUBE_URL: str = "https://www.youtube.com/watch?v=Gr7T07WfIhM"
MODEL_NAME: str = "facebook/m2m100_418M"
BATCH_SIZE: int = 4
SPACY_MODEL: str = "en_core_web_sm"
VIDEO_OUTPUT_FILE: str = os.path.join(OUTPUT_DIR, "whisper_source_video.mp4.webm")

os.makedirs(OUTPUT_DIR, exist_ok=True)

try:
    nlp = spacy.load(SPACY_MODEL)
except Exception as e:
    logger.error(f"Failed to load spaCy model '{SPACY_MODEL}': {e}")
    raise

Segment = Dict[str, Optional[float or str]]
Sentence = Dict[str, float or str]

def run_yt_dlp_audio(youtube_url: str, output_file: str) -> None:
    if not os.path.exists(output_file):
        try:
            subprocess.run([
                "yt-dlp",
                "-f", "bestaudio",
                "--extract-audio",
                "--audio-format", "wav",
                "-o", output_file,
                youtube_url
            ], check=True)
        except subprocess.CalledProcessError as e:
            logger.error(f"yt-dlp failed: {e}")
            raise

def run_yt_dlp_video(youtube_url: str, output_file: str) -> None:
    if not os.path.exists(output_file):
        format_str = "bestvideo[height<=720]+bestaudio/best[height<=720]"
        try:
            subprocess.run(
                [
                    "yt-dlp",
                    "-f", format_str,
                    "-o", output_file,
                    youtube_url
                ],
                check=True
            )
        except subprocess.CalledProcessError as e:
            logger.error(f"yt-dlp failed: {e}")
            raise

def transcribe_audio(input_file: str, output_file: str, model_size: str = "medium") -> List[Segment]:
    if os.path.exists(output_file):
        return load_transcription(output_file)
    try:
        model = whisper.load_model(model_size)
    except Exception as e:
        logger.error(f"Failed to load Whisper model: {e}")
        raise
    try:
        result = model.transcribe(input_file)
    except Exception as e:
        logger.error(f"Transcription failed: {e}")
        raise
    segments = result.get("segments", [])
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(segments, f, indent=4, ensure_ascii=False)
    logger.info(f"Transcription saved to: {output_file}")
    return segments

def load_transcription(transcription_file: str) -> List[Segment]:
    if not os.path.exists(transcription_file):
        return []
    try:
        with open(transcription_file, "r", encoding="utf-8") as f:
            segments = json.load(f)
    except Exception as e:
        logger.error(f"Failed to load transcription file '{transcription_file}': {e}")
        return []
    return segments

def segment_sentences_spacy(segments: List[Segment]) -> List[Sentence]:
    full_text = " ".join([segment["text"] for segment in segments])
    try:
        doc = nlp(full_text)
    except Exception as e:
        logger.error(f"spaCy failed during sentence segmentation: {e}")
        raise
    sentences = list(doc.sents)
    sentence_segments: List[Sentence] = []
    char_index = 0

    for idx, sentence in enumerate(sentences):
        sent_start_char = sentence.start_char
        sent_end_char = sentence.end_char
        sent_start: Optional[float] = None
        sent_end: Optional[float] = None

        for segment in segments:
            if 'cumulative_start' not in segment:
                segment['cumulative_start'] = char_index
                char_index += len(segment["text"]) + 1
                segment['cumulative_end'] = char_index

            if sent_start is None and segment['cumulative_start'] <= sent_start_char < segment['cumulative_end']:
                sent_start = segment.get("start")

            if segment['cumulative_start'] <= sent_end_char <= segment['cumulative_end']:
                sent_end = segment.get("end")
                break

        if sent_start is not None and sent_end is not None:
            sentence_segments.append({
                "start": sent_start,
                "end": sent_end,
                "text": sentence.text.strip()
            })
    try:
        with open(SENTENCE_SEGMENT_FILE, "w", encoding="utf-8") as f:
            json.dump(sentence_segments, f, indent=4, ensure_ascii=False)
    except Exception as e:
        logger.error(f"Failed to write sentence segments to '{SENTENCE_SEGMENT_FILE}': {e}")
        raise

    return sentence_segments

def translate_sentences(
        sentences: List[Sentence],
        model: M2M100ForConditionalGeneration,
        tokenizer: M2M100Tokenizer,
        batch_size: int = 4
) -> List[Sentence]:
    translated_sentences: List[Sentence] = []
    detokenizer = MosesDetokenizer(lang="ja")
    texts = [sentence["text"] for sentence in sentences]
    total = len(texts)
    for i in range(0, total, batch_size):
        batch_texts = texts[i:i + batch_size]
        try:
            tokenizer.src_lang = "en"
            encoded = tokenizer(batch_texts, return_tensors="pt", padding=True, truncation=True, max_length=512)
            translated = model.generate(**encoded, forced_bos_token_id=tokenizer.get_lang_id("ja"))
            translated_batch = tokenizer.batch_decode(translated, skip_special_tokens=True)
        except Exception as e:
            logger.error(f"Translation failed for batch starting at index {i}: {e}")
            continue

        for j, translated_text in enumerate(translated_batch):
            try:
                translated_text = detokenizer.detokenize(translated_text.split())
                translated_sentences.append({
                    "start": sentences[i + j]["start"],
                    "end": sentences[i + j]["end"],
                    "text": translated_text
                })
            except Exception as e:
                logger.error(f"Detokenization failed for sentence {i + j + 1}: {e}")
                translated_sentences.append({
                    "start": sentences[i + j]["start"],
                    "end": sentences[i + j]["end"],
                    "text": translated_text
                })

    logger.info(f"Translated {len(translated_sentences)} out of {total} sentences.")
    return translated_sentences

def normalize_japanese(translated_sentences: List[Sentence]) -> List[Sentence]:
    detokenizer = MosesDetokenizer(lang="ja")
    for sentence in translated_sentences:
        try:
            sentence["text"] = detokenizer.detokenize(sentence["text"].split())
        except Exception as e:
            logger.error(f"Normalization failed: {e}")
    return translated_sentences

def write_srt(sentences: List[Sentence], srt_file: str) -> None:
    try:
        with open(srt_file, "w", encoding="utf-8") as f:
            for idx, sentence in enumerate(sentences, start=1):
                start = format_timestamp(sentence["start"])
                end = format_timestamp(sentence["end"])
                text = sentence["text"]
                f.write(f"{idx}\n{start} --> {end}\n{text}\n\n")
    except Exception as e:
        logger.error(f"Failed to write SRT file '{srt_file}': {e}")
        raise
    logger.info(f"SRT file written successfully: {srt_file}")

def format_timestamp(seconds: float) -> str:
    millis = int(round((seconds - int(seconds)) * 1000))
    secs = int(seconds) % 60
    mins = (int(seconds) // 60) % 60
    hours = int(seconds) // 3600
    return f"{hours:02}:{mins:02}:{secs:02},{millis:03}"

def play_video_with_subtitles(video_file: str, srt_file: str) -> None:
    if not os.path.exists(video_file) or not os.path.exists(srt_file):
        return
    try:
        subprocess.run([
            "mpv",
            video_file,
            "--sub-file", srt_file
        ], check=True)
    except subprocess.CalledProcessError as e:
        logger.error(f"mpv failed to play video: {e}")
        raise

def process_translation_pipeline(
        skip_download: bool = False,
        skip_transcription: bool = False,
        skip_translation: bool = False,
        download_video: bool = False,
        play_video_flag: bool = False
) -> None:
    if not skip_download:
        run_yt_dlp_audio(WHISPER_YOUTUBE_URL, WHISPER_SOURCE_AUDIO)

    if download_video:
        run_yt_dlp_video(WHISPER_YOUTUBE_URL, VIDEO_OUTPUT_FILE)

    if skip_transcription and os.path.exists(WHISPER_TRANSCRIPTION_FILE):
        segments = load_transcription(WHISPER_TRANSCRIPTION_FILE)
    else:
        segments = transcribe_audio(WHISPER_SOURCE_AUDIO, WHISPER_TRANSCRIPTION_FILE, model_size="medium")

    if not segments:
        return

    sentences = segment_sentences_spacy(segments)
    if not sentences:
        return

    processed_sentences = sentences

    if not skip_translation:
        try:
            tokenizer = M2M100Tokenizer.from_pretrained(MODEL_NAME)
            model = M2M100ForConditionalGeneration.from_pretrained(MODEL_NAME)
        except Exception as e:
            logger.error(f"Failed to load translation model '{MODEL_NAME}': {e}")
            raise

        translated_sentences = translate_sentences(processed_sentences, model, tokenizer, batch_size=BATCH_SIZE)
        if not translated_sentences:
            return
    else:
        translated_sentences = processed_sentences

    if not skip_translation:
        translated_sentences = normalize_japanese(translated_sentences)

    write_srt(translated_sentences, FINAL_SRT_FILE)

    if play_video_flag:
        play_video_with_subtitles(VIDEO_OUTPUT_FILE, FINAL_SRT_FILE)

def main(
        skip_download: bool = False,
        skip_transcription: bool = False,
        skip_translation: bool = False,
        download_video: bool = False,
        play_video_flag: bool = False
) -> None:
    try:
        process_translation_pipeline(
            skip_download=skip_download,
            skip_transcription=skip_transcription,
            skip_translation=skip_translation,
            download_video=download_video,
            play_video_flag=play_video_flag
        )
    except Exception as e:
        logger.critical(f"Pipeline terminated due to an unexpected error: {e}", exc_info=True)

if __name__ == "__main__":
    main(
        skip_download=True,
        skip_transcription=True,
        skip_translation=False,
        download_video=False,
        play_video_flag=True
    )
