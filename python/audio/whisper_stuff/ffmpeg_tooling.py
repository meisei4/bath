import os
import json
import subprocess
import logging
from typing import List, Dict, Optional






import whisper
import spacy
from transformers import M2M100ForConditionalGeneration, M2M100Tokenizer
from sacremoses import MosesDetokenizer

# Configure logging to include INFO level
logging.basicConfig(
    level=logging.INFO,  # Set to INFO to reduce verbosity
    format='%(asctime)s [%(levelname)s] %(message)s',
    handlers=[
        logging.FileHandler("translation_pipeline.log"),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

# Configuration Variables
OUTPUT_DIR: str = os.path.join(os.getcwd(), "output_audio")
WHISPER_SOURCE_AUDIO: str = os.path.join(OUTPUT_DIR, "whisper_source_audio.wav")
WHISPER_TRANSCRIPTION_FILE: str = os.path.join(OUTPUT_DIR, "whisper_transcription.json")
SENTENCE_SEGMENT_FILE: str = os.path.join(OUTPUT_DIR, "sentence_segments.json")
FINAL_SRT_FILE: str = os.path.join(OUTPUT_DIR, "translated_subtitles.srt")
WHISPER_YOUTUBE_URL: str = "https://www.youtube.com/watch?v=Gr7T07WfIhM"
MODEL_NAME: str = "facebook/m2m100_418M"  # Updated Translation Model
BATCH_SIZE: int = 4  # Adjusted Batch Size for flexibility
SPACY_MODEL: str = "en_core_web_sm"
VIDEO_OUTPUT_FILE: str = os.path.join(OUTPUT_DIR, "whisper_source_video.mp4.webm")

# Ensure output directory exists
os.makedirs(OUTPUT_DIR, exist_ok=True)
logger.info(f"Output directory set at: {OUTPUT_DIR}")

# Load spaCy model
try:
    logger.info(f"Loading spaCy model: {SPACY_MODEL}")
    nlp = spacy.load(SPACY_MODEL)
    logger.info(f"Loaded spaCy model: {SPACY_MODEL}")
except Exception as e:
    logger.error(f"Failed to load spaCy model '{SPACY_MODEL}': {e}")
    raise

# Type Definitions
Segment = Dict[str, Optional[float or str]]
Sentence = Dict[str, float or str]


def run_yt_dlp_audio(youtube_url: str, output_file: str) -> None:
    """
    Downloads the audio from a YouTube video using yt-dlp.
    """
    logger.info(f"Checking if audio file exists: {output_file}")
    if os.path.exists(output_file):
        logger.info(f"Audio file already exists: {output_file}")
        return
    logger.info(f"Downloading audio from: {youtube_url}")
    try:
        subprocess.run([
            "yt-dlp",
            "-f", "bestaudio",
            "--extract-audio",
            "--audio-format", "wav",
            "-o", output_file,
            youtube_url
        ], check=True)
        logger.info(f"Downloaded audio to: {output_file}")
    except subprocess.CalledProcessError as e:
        logger.error(f"yt-dlp failed: {e}")
        raise

def run_yt_dlp_video(youtube_url: str, output_file: str) -> None:
    logger.info(f"Checking if video file exists: {output_file}")
    if os.path.exists(output_file):
        logger.info(f"Video file already exists: {output_file}")
        return

    logger.info(f"Downloading 720p video from: {youtube_url}")

    # Define the format string for 720p
    format_str = "bestvideo[height<=720]+bestaudio/best[height<=720]"

    try:
        subprocess.run(
            [
                "yt-dlp",
                "-f", format_str,  # Specify the format for 720p
                "-o", output_file,
                youtube_url
            ],
            check=True
        )
        logger.info(f"Downloaded video to: {output_file}")
    except subprocess.CalledProcessError as e:
        logger.error(f"yt-dlp failed: {e}")
        raise


def transcribe_audio(input_file: str, output_file: str, model_size: str = "medium") -> List[Segment]:
    """
    Transcribes audio using Whisper and saves the transcription to a JSON file.
    """
    logger.info(f"Checking if transcription file exists: {output_file}")
    if os.path.exists(output_file):
        logger.info(f"Transcription file already exists: {output_file}")
        return load_transcription(output_file)
    logger.info(f"Loading Whisper model (size: {model_size})")
    try:
        model = whisper.load_model(model_size)
        logger.info("Whisper model loaded successfully.")
    except Exception as e:
        logger.error(f"Failed to load Whisper model: {e}")
        raise
    logger.info(f"Transcribing audio file: {input_file}")
    try:
        result = model.transcribe(input_file)
        logger.info("Transcription completed.")
    except Exception as e:
        logger.error(f"Transcription failed: {e}")
        raise
    segments = result.get("segments", [])
    logger.info(f"Number of segments transcribed: {len(segments)}")
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(segments, f, indent=4, ensure_ascii=False)
    logger.info(f"Transcription saved to: {output_file}")
    return segments


def load_transcription(transcription_file: str) -> List[Segment]:
    """
    Loads the transcription from a JSON file.
    """
    logger.info(f"Loading transcription from: {transcription_file}")
    if not os.path.exists(transcription_file):
        logger.warning(f"Transcription file not found: {transcription_file}")
        return []
    try:
        with open(transcription_file, "r", encoding="utf-8") as f:
            segments = json.load(f)
        logger.info(f"Loaded {len(segments)} transcription segments.")
    except Exception as e:
        logger.error(f"Failed to load transcription file '{transcription_file}': {e}")
        return []
    return segments


def segment_sentences_spacy(segments: List[Segment]) -> List[Sentence]:
    """
    Segments transcription into sentences using spaCy.
    """
    logger.info("Segmenting transcription into sentences.")
    full_text = " ".join([segment["text"] for segment in segments])
    logger.info(f"Full transcription text length: {len(full_text)} characters.")
    try:
        doc = nlp(full_text)
        logger.info("Completed sentence segmentation using spaCy.")
    except Exception as e:
        logger.error(f"spaCy failed during sentence segmentation: {e}")
        raise
    sentences = list(doc.sents)
    logger.info(f"Identified {len(sentences)} sentences.")
    sentence_segments: List[Sentence] = []
    char_index = 0

    for idx, sentence in enumerate(sentences):
        logger.info(f"Processing sentence {idx + 1}/{len(sentences)}")
        sent_start_char = sentence.start_char
        sent_end_char = sentence.end_char
        sent_start: Optional[float] = None
        sent_end: Optional[float] = None

        for segment in segments:
            if 'cumulative_start' not in segment:
                segment['cumulative_start'] = char_index
                char_index += len(segment["text"]) + 1  # +1 for space
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
            logger.info(f"Appended sentence {idx + 1}: Start={sent_start}, End={sent_end}")
        else:
            logger.warning(f"Could not determine timing for sentence {idx + 1}: '{sentence.text.strip()}'")

    try:
        with open(SENTENCE_SEGMENT_FILE, "w", encoding="utf-8") as f:
            json.dump(sentence_segments, f, indent=4, ensure_ascii=False)
        logger.info(f"Segmented into {len(sentence_segments)} sentences.")
    except Exception as e:
        logger.error(f"Failed to write sentence segments to '{SENTENCE_SEGMENT_FILE}': {e}")
        raise

    return sentence_segments


def translate_sentences(
        sentences: List[Sentence],
        model: M2M100ForConditionalGeneration,
        tokenizer: M2M100Tokenizer,
        batch_size: int = 4  # Updated to match the global BATCH_SIZE variable
) -> List[Sentence]:
    """
    Translates sentences into Japanese using the M2M100 model.
    """
    logger.info("Starting translation of sentences.")
    translated_sentences: List[Sentence] = []
    detokenizer = MosesDetokenizer(lang="ja")
    texts = [sentence["text"] for sentence in sentences]
    total = len(texts)
    total_batches = (total + batch_size - 1) // batch_size
    logger.info(f"Total sentences to translate: {total} in {total_batches} batches.")

    for i in range(0, total, batch_size):
        batch_texts = texts[i:i + batch_size]
        batch_number = (i // batch_size) + 1
        logger.info(f"Translating batch {batch_number}/{total_batches} with {len(batch_texts)} sentences.")
        try:
            # Set the source language to English
            tokenizer.src_lang = "en"
            encoded = tokenizer(batch_texts, return_tensors="pt", padding=True, truncation=True, max_length=512)
            # Generate translation with target language set to Japanese
            translated = model.generate(**encoded, forced_bos_token_id=tokenizer.get_lang_id("ja"))
            translated_batch = tokenizer.batch_decode(translated, skip_special_tokens=True)
            logger.info(f"Completed translation for batch {batch_number}.")
        except Exception as e:
            logger.error(f"Translation failed for batch {batch_number}: {e}")
            continue

        for j, translated_text in enumerate(translated_batch):
            try:
                translated_text = detokenizer.detokenize(translated_text.split())
                translated_sentences.append({
                    "start": sentences[i + j]["start"],
                    "end": sentences[i + j]["end"],
                    "text": translated_text
                })
                # Log every 10th translated sentence for sampling
                if (i + j + 1) % 10 == 0:
                    logger.info(f"Sample Translated sentence {i + j + 1}: '{translated_text}'")
            except Exception as e:
                logger.error(f"Detokenization failed for sentence {i + j + 1}: {e}")
                translated_sentences.append({
                    "start": sentences[i + j]["start"],
                    "end": sentences[i + j]["end"],
                    "text": translated_text  # Fallback to original translation
                })

    logger.info(f"Translated {len(translated_sentences)} out of {total} sentences.")
    return translated_sentences


def normalize_japanese(translated_sentences: List[Sentence]) -> List[Sentence]:
    """
    Normalizes Japanese text using MosesDetokenizer.
    """
    logger.info("Normalizing Japanese text in translated sentences.")
    detokenizer = MosesDetokenizer(lang="ja")
    for idx, sentence in enumerate(translated_sentences):
        try:
            original_text = sentence["text"]
            sentence["text"] = detokenizer.detokenize(sentence["text"].split())
            logger.info(f"Normalized sentence {idx + 1}: '{original_text}' -> '{sentence['text']}'")
        except Exception as e:
            logger.error(f"Normalization failed for sentence {idx + 1}: {e}")
    logger.info("Japanese text normalization complete.")
    return translated_sentences


def write_srt(sentences: List[Sentence], srt_file: str) -> None:
    """
    Writes translated sentences to an SRT file.
    """
    logger.info(f"Writing subtitles to SRT file: {srt_file}")
    try:
        with open(srt_file, "w", encoding="utf-8") as f:
            for idx, sentence in enumerate(sentences, start=1):
                start = format_timestamp(sentence["start"])
                end = format_timestamp(sentence["end"])
                text = sentence["text"]
                f.write(f"{idx}\n{start} --> {end}\n{text}\n\n")
                logger.info(f"Wrote subtitle {idx}: {start} --> {end}")
        logger.info(f"SRT file written successfully: {srt_file}")
    except Exception as e:
        logger.error(f"Failed to write SRT file '{srt_file}': {e}")
        raise


def format_timestamp(seconds: float) -> str:
    """
    Formats seconds to SRT timestamp format.
    """
    millis = int(round((seconds - int(seconds)) * 1000))
    secs = int(seconds) % 60
    mins = (int(seconds) // 60) % 60
    hours = int(seconds) // 3600
    timestamp = f"{hours:02}:{mins:02}:{secs:02},{millis:03}"
    return timestamp


def play_video_with_subtitles(video_file: str, srt_file: str) -> None:
    """
    Plays the video with subtitles using mpv.
    """
    logger.info(f"Playing video {video_file} with subtitles {srt_file} using mpv.")
    if not os.path.exists(video_file):
        logger.error(f"Video file not found: {video_file}")
        return
    if not os.path.exists(srt_file):
        logger.error(f"SRT file not found: {srt_file}")
        return
    try:
        subprocess.run([
            "mpv",
            video_file,
            "--sub-file", srt_file
        ], check=True)
        logger.info("Video playback completed.")
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
    """
    Orchestrates the translation pipeline based on provided flags.
    """
    logger.info("Starting translation pipeline.")

    # Step 1: Download Audio
    if not skip_download:
        logger.info("Downloading audio as skip_download is False.")
        run_yt_dlp_audio(WHISPER_YOUTUBE_URL, WHISPER_SOURCE_AUDIO)
    else:
        logger.info("Skipping audio download as per configuration.")

    # Step 2: Download Video (Optional)
    if download_video:
        logger.info("Downloading video as per configuration.")
        run_yt_dlp_video(WHISPER_YOUTUBE_URL, VIDEO_OUTPUT_FILE)
    else:
        logger.info("Skipping video download as per configuration.")

    # Step 3: Transcribe Audio
    if skip_transcription and os.path.exists(WHISPER_TRANSCRIPTION_FILE):
        logger.info("Skipping transcription as per configuration and transcription file exists.")
        segments = load_transcription(WHISPER_TRANSCRIPTION_FILE)
    else:
        logger.info("Performing transcription as per configuration.")
        segments = transcribe_audio(WHISPER_SOURCE_AUDIO, WHISPER_TRANSCRIPTION_FILE, model_size="medium")

    if not segments:
        logger.warning("No transcription segments available. Exiting pipeline.")
        return

    # Step 4: Segment Sentences
    sentences = segment_sentences_spacy(segments)
    if not sentences:
        logger.warning("No sentences segmented. Exiting pipeline.")
        return

    processed_sentences = sentences

    # Step 5: Translate Sentences
    if not skip_translation:
        try:
            logger.info(f"Loading translation tokenizer and model: {MODEL_NAME}")
            tokenizer = M2M100Tokenizer.from_pretrained(MODEL_NAME)
            model = M2M100ForConditionalGeneration.from_pretrained(MODEL_NAME)
            logger.info(f"Loaded translation model and tokenizer: {MODEL_NAME}")
        except Exception as e:
            logger.error(f"Failed to load translation model '{MODEL_NAME}': {e}")
            raise

        translated_sentences = translate_sentences(processed_sentences, model, tokenizer, batch_size=BATCH_SIZE)
        if not translated_sentences:
            logger.warning("No sentences were translated. Exiting pipeline.")
            return
    else:
        logger.info("Skipping translation as per configuration.")
        # If skipping translation, use the original sentences
        translated_sentences = processed_sentences

    # Step 6: Normalize Japanese Text
    if not skip_translation:
        translated_sentences = normalize_japanese(translated_sentences)

    # Step 7: Write SRT File
    write_srt(translated_sentences, FINAL_SRT_FILE)

    logger.info("Translation pipeline completed successfully.")

    # Step 8: Play Video with Subtitles (Optional)
    if play_video_flag:
        logger.info("Initiating video playback as per configuration.")
        play_video_with_subtitles(VIDEO_OUTPUT_FILE, FINAL_SRT_FILE)
    else:
        logger.info("Skipping video playback as per configuration.")


def main(
        skip_download: bool = False,
        skip_transcription: bool = False,
        skip_translation: bool = False,
        download_video: bool = False,
        play_video_flag: bool = False
) -> None:
    """
    Initiates the translation pipeline with specified flags.
    """
    logger.info("Pipeline execution started.")
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
        skip_download=True,         # Set to True to skip audio download
        skip_transcription=True,    # Set to True to skip transcription
        skip_translation=False,      # Set to True to skip translation
        download_video=False,         # Set to True to download the video
        play_video_flag=True         # Set to True to play the video with subtitles
    )
