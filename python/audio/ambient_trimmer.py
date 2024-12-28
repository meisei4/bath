import os
import subprocess
import json

# Configuration
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
INPUT_AUDIO_DIR = os.path.join(SCRIPT_DIR, "../assets/ambient")
OUTPUT_AUDIO_DIR = os.path.join(SCRIPT_DIR, "output/")
MERGED_AUDIO_FILE = os.path.join(OUTPUT_AUDIO_DIR, "full_album_merged.mp3")
METADATA_FILE = os.path.join(OUTPUT_AUDIO_DIR, "track_metadata.json")
FINAL_TRIMMED_FILE = os.path.join(OUTPUT_AUDIO_DIR, "final_trimmed_album.mp3")
TRIM_METADATA_FILE = os.path.join(OUTPUT_AUDIO_DIR, "trim_metadata.json")

# Updated TRIMS Configuration
TRIMS = [
    {"track_name": "01 - Only Shallow", "relative_start_time": "00:03:40"},
    {"track_name": "03 - Touched", "relative_start_time": "00:00:00"},
    {"track_name": "04 - To Here Knows When", "relative_start_time": "00:04:46"},
    {"track_name": "05 - When You Sleep", "relative_start_time": "00:03:57"},
    {
        "track_name": "10 - What You Want",
        "relative_start_time": "00:04:21",
        "cross_duration": "00:00:02"  # Extend 2 seconds into the next track
    }
]

# Ensure output directory exists
os.makedirs(OUTPUT_AUDIO_DIR, exist_ok=True)

def run_ffmpeg(input_files, output_file):
    """Concatenates input_files into output_file using ffmpeg."""
    temp_file = os.path.join(SCRIPT_DIR, "file_list.txt")
    with open(temp_file, "w") as f:
        f.writelines(f"file '{file}'\n" for file in input_files)

    subprocess.run([
        "ffmpeg", "-f", "concat", "-safe", "0", "-i", temp_file, "-c", "copy", output_file
    ], check=True)
    os.remove(temp_file)
    print(f"Merged audio saved to: {output_file}")

def generate_metadata(input_files):
    metadata = []
    current_start = 0.0

    for file in input_files:
        result = subprocess.run([
            "ffprobe", "-v", "error", "-show_entries",
            "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", file
        ], capture_output=True, text=True, check=True)

        duration = float(result.stdout.strip())
        track_name = os.path.splitext(os.path.basename(file))[0]
        metadata.append({
            "file": file,
            "track_name": track_name,
            "start_time": current_start,
            "end_time": current_start + duration
        })
        current_start += duration

    with open(METADATA_FILE, "w") as f:
        json.dump(metadata, f, indent=4)
    print(f"Metadata saved to: {METADATA_FILE}")

    return metadata

def trim_audio(input_file, output_file, start_time, end_time):
    command = [
        "ffmpeg",
        "-i", input_file,
        "-ss", start_time,
        "-to", end_time,
        "-c", "copy",
        output_file
    ]

    try:
        subprocess.run(command, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        print(f"Trimmed audio saved to: {output_file}")
    except subprocess.CalledProcessError as e:
        stderr = e.stderr.decode()
        print(f"Error during trimming: {stderr}. Command: {' '.join(command)}")
        raise e  # Re-raise exception to handle it in the main flow

def time_str_to_seconds(time_str):
    parts = time_str.strip().split(':')
    if len(parts) != 3:
        raise ValueError(f"Invalid time format: {time_str}. Expected HH:MM:SS")
    hours, minutes, seconds = map(float, parts)
    return hours * 3600 + minutes * 60 + seconds

def seconds_to_time_str(seconds):
    hours = int(seconds // 3600)
    minutes = int((seconds % 3600) // 60)
    secs = int(seconds % 60)
    return f"{hours:02}:{minutes:02}:{secs:02}"

def process_tracks(metadata):
    trim_metadata = []
    trimmed_files = []
    merged_duration = metadata[-1]["end_time"]

    for trim in TRIMS:
        track = next((t for t in metadata if t["track_name"] == trim["track_name"]), None)

        if not track:
            print(f"Track '{trim['track_name']}' not found in metadata. Skipping.")
            continue

        # Convert relative_start_time to seconds
        try:
            relative_start_seconds = time_str_to_seconds(trim["relative_start_time"])
        except ValueError as ve:
            print(f"Invalid relative_start_time format for '{trim['track_name']}': {ve}. Skipping.")
            continue

        # Calculate global start_time
        global_start_seconds = track["start_time"] + relative_start_seconds
        if global_start_seconds >= merged_duration:
            print(f"Global start_time {seconds_to_time_str(global_start_seconds)} for '{trim['track_name']}' exceeds album duration. Skipping.")
            continue

        # Calculate global end_time
        if "cross_duration" in trim:
            try:
                cross_duration_seconds = time_str_to_seconds(trim["cross_duration"])
            except ValueError as ve:
                print(f"Invalid cross_duration format for '{trim['track_name']}': {ve}. Skipping.")
                continue
            global_end_seconds = track["end_time"] + cross_duration_seconds
            # Ensure end_time does not exceed merged album's duration
            global_end_seconds = min(global_end_seconds, merged_duration)
        else:
            # Trim until the end of the track
            global_end_seconds = track["end_time"]

        global_end_time = seconds_to_time_str(global_end_seconds)
        global_start_time = seconds_to_time_str(global_start_seconds)

        # Create output file name
        sanitized_track_name = trim["track_name"].replace(" ", "_").replace("-", "_")
        output_file = os.path.join(
            OUTPUT_AUDIO_DIR,
            f"trimmed_{sanitized_track_name}_{global_start_time.replace(':', '-')}_{global_end_time.replace(':', '-')}.mp3"
        )

        # Perform trimming
        try:
            trim_audio(MERGED_AUDIO_FILE, output_file, global_start_time, global_end_time)
            trimmed_files.append(output_file)

            trim_metadata.append({
                "track_name": trim["track_name"],
                "relative_start_time": trim["relative_start_time"],
                "cross_duration": trim.get("cross_duration", "00:00:00"),
                "global_start_time": global_start_time,
                "global_end_time": global_end_time,
                "output_file": output_file
            })
        except subprocess.CalledProcessError:
            print(f"Failed to trim '{trim['track_name']}'. Continuing with next trim.")
            continue

    # Save trim metadata
    with open(TRIM_METADATA_FILE, "w") as f:
        json.dump(trim_metadata, f, indent=4)
    print(f"Trim metadata saved to: {TRIM_METADATA_FILE}")

    return trimmed_files

def merge_trimmed_files(trimmed_files):
    if not trimmed_files:
        print("No trimmed files to merge.")
        return
    try:
        run_ffmpeg(trimmed_files, FINAL_TRIMMED_FILE)
        print(f"Final trimmed album saved to: {FINAL_TRIMMED_FILE}")
    except subprocess.CalledProcessError as e:
        print(f"Error merging trimmed files: {e}")
        raise e

def main():
    # List all MP3 files in the input directory, sorted
    mp3_files = [
        os.path.join(INPUT_AUDIO_DIR, file)
        for file in sorted(os.listdir(INPUT_AUDIO_DIR))
        if file.lower().endswith(".mp3")
    ]

    if not mp3_files:
        print("No MP3 files found in the input directory.")
        return

    print("Merging audio files...")
    run_ffmpeg(mp3_files, MERGED_AUDIO_FILE)

    print("Generating metadata...")
    metadata = generate_metadata(mp3_files)

    print("Processing trims...")
    trimmed_files = process_tracks(metadata)

    print("Merging trimmed files...")
    merge_trimmed_files(trimmed_files)

if __name__ == "__main__":
    main()
