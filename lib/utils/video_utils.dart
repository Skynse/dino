/// video_utils.dart
/// Utility functions and helpers for video editing operations.
/// Add your video processing, formatting, and helper methods here.
library;

class VideoUtils {
  /// Formats a duration (in seconds) to mm:ss string.
  static String formatDuration(Duration duration) {
    String twoDigits(int n) => n.toString().padLeft(2, '0');
    final minutes = twoDigits(duration.inMinutes.remainder(60));
    final seconds = twoDigits(duration.inSeconds.remainder(60));
    return "$minutes:$seconds";
  }

  // Add more video-related utility methods as needed.
}
