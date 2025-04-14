extends Node

# Global dictionary to hold metrics.
# Each entry holds:
#   "calls": number of times the function is instrumented,
#   "total_time_usec": total elapsed microseconds over all calls,
#   "extra": an array of dictionaries of extra context-specific values per call.
var metrics = {}  # metric_name -> { "calls": int, "total_time_usec": int, "extra": Array }

# Record a measurement for a given metric.
# extra_info is an optional dictionary that you can pass context-specific data.
func record(metric_name: String, elapsed_usec: int, extra_info: Dictionary = {}) -> void:
    if not metrics.has(metric_name):
        metrics[metric_name] = {"calls": 0, "total_time_usec": 0, "extra": []}
    metrics[metric_name]["calls"] += 1
    metrics[metric_name]["total_time_usec"] += elapsed_usec
    metrics[metric_name]["extra"].append(extra_info)

# Reset all stored metrics.
func reset_metrics() -> void:
    metrics.clear()

# Print a snapshot of the current metrics.
# Set cumulative to true if you want to accumulate metrics over multiple prints;
# otherwise, metrics are reset after printing.
func print_metrics(cumulative: bool = false) -> void:
    var current_frame = Engine.get_frames_drawn()
    var current_time = Time.get_ticks_msec()
    print("----- Profiling Snapshot at %d ms (Frame: %d) -----" % [current_time, current_frame])
    print("Metric,Calls,TotalTime(µs),AvgTime(µs),Details")
    for name in metrics.keys():
        var data = metrics[name]
        var avg = data["total_time_usec"] / data["calls"]
        var detail_str = ""
        # For each call, list its extra context information in a compact format.
        for extra in data["extra"]:
            var pairs = []
            for key in extra.keys():
                pairs.append("%s: %s" % [key, str(extra[key])])
            detail_str += "(" + ", ".join(pairs) + ") "
        print("%s,%d,%d,%.2f,%s" % [name, data["calls"], data["total_time_usec"], avg, detail_str])
    print("----------------------------------------------------------")
    if not cumulative:
        reset_metrics()

# Utility function to start a timer.
func start_timer() -> int:
    return Time.get_ticks_usec()
