import 'package:flutter/foundation.dart';

/// Enum for media types supported in the editor.
enum MediaType { video, audio }

/// Represents a minimal video project structure.
class VideoProject {
  final String title;
  final List<VideoClip> clips;

  VideoProject({required this.title, required this.clips});
}

/// Represents a single media clip (video or audio) in the project.
class VideoClip {
  final String id;
  final String filePath;
  final Duration start;
  final Duration end;
  final MediaType type;

  VideoClip({
    required this.id,
    required this.filePath,
    required this.start,
    required this.end,
    required this.type,
  });
}
