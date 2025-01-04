import os
import subprocess

# Constants
OUTPUT_DIR: str = os.path.join(os.getcwd(), "output_audio")
SONG_SOURCE_AUDIO: str = os.path.join(OUTPUT_DIR, "song_source_audio.wav")
SONG_OUTPUT_FILE: str = os.path.join(OUTPUT_DIR, "song_trimmed_audio.wav")
SONG_YOUTUBE_URL: str = "https://www.youtube.com/watch?v=APTUCCUbLEM"
SONG_START_TIME: str = "00:03"
SONG_DURATION: str = "00:01:19"  # 1 minute and 19 seconds

os.makedirs(OUTPUT_DIR, exist_ok=True)


def run_yt_dlp(youtube_url: str, output_file: str) -> None:
    """Downloads audio from YouTube using yt-dlp."""
    subprocess.run([
        "yt-dlp",
        "-f", "bestaudio",
        "--extract-audio",
        "--audio-format", "wav",
        "-o", output_file,
        youtube_url
    ], check=True)
    print(f"Audio downloaded to {output_file}")


def run_ffmpeg_trim(input_file: str, output_file: str, start_time: str, duration: str) -> None:
    """Trims audio using FFmpeg."""
    subprocess.run([
        "ffmpeg", "-i", input_file,
        "-ss", start_time, "-t", duration,
        "-c", "copy", output_file
    ], check=True)
    print(f"Trimmed audio saved to {output_file}")


def play_audio(audio_file: str) -> None:
    """Plays the audio file on a loop using system utilities."""
    print(f"Playing audio on loop: {audio_file}")
    if os.name == "posix":
        os.system(f"while true; do afplay {audio_file}; done")  # For macOS
    elif os.name == "nt":
        os.system(f':loop & start /b {audio_file} & timeout /t 1 & goto loop')  # For Windows
    else:
        os.system(f"while true; do aplay {audio_file}; done")  # For Linux


def process_song_trim() -> None:
    """Handles the song trimming workflow."""
    run_yt_dlp(SONG_YOUTUBE_URL, SONG_SOURCE_AUDIO)
    run_ffmpeg_trim(SONG_SOURCE_AUDIO, SONG_OUTPUT_FILE, SONG_START_TIME, SONG_DURATION)
    play_audio(SONG_OUTPUT_FILE)


if __name__ == "__main__":
    process_song_trim()
