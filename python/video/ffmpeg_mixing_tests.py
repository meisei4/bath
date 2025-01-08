import os
import sys
import subprocess

def run_cmd(cmd, desc):
    """Runs a shell command with logging. Exits if error occurs."""
    print(f"--- {desc} ---")
    try:
        subprocess.run(cmd, check=True)
        print(f"[OK] {desc}\n")
    except subprocess.CalledProcessError as e:
        print(f"[ERROR] {desc}\n{e}")
        sys.exit(1)

def process_video(
    excerpt_path,          # /Users/mac/godot_projects/bath/output_audio/excerpt.mp4
    melt_banana_path,      # /Users/mac/godot_projects/bath/output_audio/song_trimmed_audio.wav
    youtube_url,           # https://www.youtube.com/watch?v=_ypjBAIz0EQ
    final_output,          # /Users/mac/godot_projects/bath/output_audio/final_output.mp4
    overlay_start_sec=18,  # Melt-Banana at 0:18
    overlay_volume=0.05,   # 5% volume
    glitch_start=85,       # Glitch from 85–88s
    glitch_dur=3,
    final_length=300,      # Extend to 300s total
    youtube_fade_start=70, # YouTube track appears at 70s
    youtube_fade_dur=10    # Fade from 0.03 → 0.4 over 10s
):
    """
    Steps:
      1) Overlay Melt-Banana track @ 18s, volume=0.05
      2) Glitch 85–88s (not just random static, using mild TBlend effect)
      3) Freeze last frame from 88s → 300s
      4) Download full YouTube track, fade in from 0.03→0.4 between 70–80s
      5) Amix final video audio w/ YouTube track -> final_output (300s)
    """

    # ----------------------
    # Validate input files
    # ----------------------
    if not os.path.isfile(excerpt_path):
        print(f"[ERROR] excerpt not found: {excerpt_path}")
        sys.exit(1)
    if not os.path.isfile(melt_banana_path):
        print(f"[ERROR] Melt-Banana track not found: {melt_banana_path}")
        sys.exit(1)

    out_dir = os.path.dirname(final_output)
    os.makedirs(out_dir, exist_ok=True)

    # Intermediate file paths
    step1_overlay = os.path.join(out_dir, "tmp_step1_overlay.mp4")  # after Melt-Banana
    step2_glitch  = os.path.join(out_dir, "tmp_step2_glitch.mp4")   # after mild glitch
    step3_extend  = os.path.join(out_dir, "tmp_step3_extend.mp4")   # extended to 300s
    youtube_audio = os.path.join(out_dir, "youtube_full.wav")

    # 1) Overlay Melt-Banana at 0:18, volume=0.05
    delay_ms = int(overlay_start_sec * 1000)
    run_cmd(
        [
            "ffmpeg", "-y",
            "-i", excerpt_path,
            "-i", melt_banana_path,
            "-filter_complex",
            (
                f"[1:a]adelay={delay_ms}|{delay_ms},volume={overlay_volume}[melt];"
                f"[0:a][melt]amix=inputs=2:duration=first[aout]"
            ),
            "-map", "0:v",
            "-map", "[aout]",
            "-c:v", "copy",
            "-c:a", "aac",
            "-shortest",
            step1_overlay
        ],
        "Overlaying Melt-Banana track"
    )

    # 2) Mild glitch from 85–88s using TBlend difference
    #    We'll do a split: main portion 0–85, glitch portion 85–88 w/ TBlend
    glitch_filter = (
        f"[0:v]split=2[vmain][vraw];"
        f"[vmain]trim=0:{glitch_start},setpts=PTS-STARTPTS[v1];"
        f"[vraw]trim={glitch_start}:{glitch_start+glitch_dur},setpts=PTS-STARTPTS,"
        # mild difference glitch
        f"tblend=all_mode=difference[vg];"
        #
        f"[v1][vg]concat=n=2:v=1:a=0[vout];"

        # For audio: half-volume on glitch portion
        f"[0:a]asplit=2[amain][araw];"
        f"[amain]atrim=0:{glitch_start},asetpts=PTS-STARTPTS[a1];"
        f"[araw]atrim={glitch_start}:{glitch_start+glitch_dur},asetpts=PTS-STARTPTS,"
        f"volume=0.5[ag];"
        f"[a1][ag]concat=v=0:a=1[aout]"
    )

    run_cmd(
        [
            "ffmpeg", "-y",
            "-i", step1_overlay,
            "-filter_complex", glitch_filter,
            "-map", "[vout]",
            "-map", "[aout]",
            "-c:v", "libx264",
            "-crf", "18",
            "-pix_fmt", "yuv420p",
            "-c:a", "aac",
            step2_glitch
        ],
        "Applying mild glitch for 3s"
    )

    # 3) Freeze final frame from 88s → 300s
    #    We'll do:
    #    - first part: 0–88
    #    - freeze last frame from 88–300
    freeze_filter = (
        f"[0:v]trim=0:{glitch_start+glitch_dur},setpts=PTS-STARTPTS[vpart];"
        # Freeze last frame
        f"[0:v]trim=start={glitch_start+glitch_dur}:end={glitch_start+glitch_dur},"
        f"setpts=PTS-STARTPTS[f];"
        f"[f]tpad=start_duration={final_length - (glitch_start+glitch_dur)}[vfreeze];"
        f"[vpart][vfreeze]concat=n=2:v=1[a=0][vout];"  # 'a=0' just means no audio from concat

        # For audio, do similarly but just hold silence or last sample
        f"[0:a]atrim=0:{glitch_start+glitch_dur},asetpts=PTS-STARTPTS[apart];"
        # We'll freeze last audio frame as silence. Easiest is to use an aevalsrc=0.0.
        # Another approach is tpad, but that can glitch. Let's do silent extension:
        f"aevalsrc=0:d={final_length - (glitch_start+glitch_dur)}[asilent];"
        f"[apart][asilent]concat=v=0:a=1[aout]"
    )

    # We can't have two outputs in one chain easily, so let's simplify with a two-step filter:
    # We'll do a short filter that first extracts 0–88, freeze last frame of video, silence last portion of audio
    freeze_filter_simplified = (
        f"[0:v]trim=0:{glitch_start+glitch_dur},setpts=PTS-STARTPTS[v1];"
        f"[v1]split=2[vkeep][vlast];"
        f"[vlast]trim=start={glitch_start+glitch_dur - (glitch_start+glitch_dur)}:"
        f"end={glitch_start+glitch_dur - (glitch_start+glitch_dur)+0.000001},setpts=PTS-STARTPTS[vlast2];"
        # Actually, easier to do "select=eq(n\,prev_n)" or tpad?
        f"[vlast]tpad=start_duration={final_length - (glitch_start+glitch_dur)}[vfreeze];"
        f"[vkeep][vfreeze]concat=n=2[vout];"

        # Audio portion
        f"[0:a]atrim=0:{glitch_start+glitch_dur},asetpts=PTS-STARTPTS[a1];"
        f"aevalsrc=0:d={final_length - (glitch_start+glitch_dur)}[asilent];"
        f"[a1][asilent]concat=v=0:a=1[aout]"
    )

    # Instead of fancy multi-split, let's do a simpler approach with "tpad":
    freeze_filter_simplified = (
        f"[0:v]trim=0:{glitch_start+glitch_dur},setpts=PTS-STARTPTS," 
        f"tpad=start_duration={final_length - (glitch_start+glitch_dur)}:start_mode=clone[vout];"

        f"[0:a]atrim=0:{glitch_start+glitch_dur},asetpts=PTS-STARTPTS,"
        f"tpad=start_duration={final_length - (glitch_start+glitch_dur)}:start_mode=clone[aout]"
    )

    run_cmd(
        [
            "ffmpeg", "-y",
            "-i", step2_glitch,
            "-filter_complex", freeze_filter_simplified,
            "-map", "[vout]",
            "-map", "[aout]",
            "-c:v", "libx264",
            "-crf", "18",
            "-pix_fmt", "yuv420p",
            "-c:a", "aac",
            step3_extend
        ],
        f"Freezing last frame -> {final_length}s total"
    )

    # 4) Download the entire YouTube track if missing
    if not os.path.isfile(youtube_audio):
        run_cmd(
            [
                "yt-dlp",
                "--extract-audio",
                "--audio-format", "wav",
                "-o", youtube_audio,
                youtube_url
            ],
            "Downloading full YouTube track"
        )
    else:
        print("[SKIP] YouTube audio already exists.\n")

    # 5) Fade in from volume=0.03→0.4 over 10s starting at 70s
    #    We'll apply:
    #       adelay=70000ms so it starts at 70s
    #       then volume expr: if(t < 70, 0), if(70<=t<80 => 0.03 + 0.37*(t-70)/10 ), else => 0.4
    #    But we can unify a bit: start at 70 => we only care about t' = t-70
    #    We'll do a simple filter: volume='if(lt(t,70),0, if(lt(t,80), 0.03+(0.37*(t-70)/10), 0.4))'
    fade_expr = (
        "if(lt(t,70), 0,"
        "if(lt(t,80), 0.03+(0.37*(t-70)/10),"
        "0.4))"
    )
    # We'll apply adelay=70000 => set your track to start at t=70
    # Then apply volume with the expression.
    # Then we do amix with [0:a].
    final_filter = (
        f"[0:a][1:a]amix=inputs=2:duration=longest[aout]"
    )

    run_cmd(
        [
            "ffmpeg", "-y",
            "-i", step3_extend,            # video+audio (300s)
            "-i", youtube_audio,
            "-filter_complex",
            (
                f"[1:a]adelay=70000|70000,volume='{fade_expr}'[yfade];"
                f"[0:a][yfade]amix=inputs=2:duration=longest[aout]"
            ),
            "-map", "0:v",
            "-map", "[aout]",
            "-c:v", "copy",     # no need to re-encode video again
            "-c:a", "aac",
            "-shortest",
            final_output
        ],
        "Final amix with slow fade-in YouTube track"
    )

    print(f"\nAll done! Final output at: {final_output}")

if __name__ == "__main__":
    EXCERPT   = "/Users/mac/godot_projects/bath/output_audio/excerpt.mp4"
    MELT_BNB  = "/Users/mac/godot_projects/bath/output_audio/song_trimmed_audio.wav"
    YT_URL    = "https://www.youtube.com/watch?v=_ypjBAIz0EQ"
    FINAL_OUT = "/Users/mac/godot_projects/bath/output_audio/final_output.mp4"

    process_video(
        excerpt_path=EXCERPT,
        melt_banana_path=MELT_BNB,
        youtube_url=YT_URL,
        final_output=FINAL_OUT,
        overlay_start_sec=18.0,
        overlay_volume=0.05,
        glitch_start=85.0,
        glitch_dur=3.0,
        final_length=300,      # Enough to cover the 3:50 track
        youtube_fade_start=70, # YouTube enters at 1:10
        youtube_fade_dur=10    # Fades from 0.03->0.4 over 10s
    )
