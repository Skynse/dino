import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'video_project.dart';

/// Enum for media types.
enum MediaType { video, audio }

/// Represents a media clip (video or audio) in the project.
class MediaClip {
  final String id;
  final String filePath;
  final Duration start;
  final Duration end;
  final MediaType type;

  MediaClip({
    required this.id,
    required this.filePath,
    required this.start,
    required this.end,
    required this.type,
  });
}

/// Represents the state of the video project.
class ProjectState {
  final String title;
  final List<MediaClip> clips;

  ProjectState({required this.title, required this.clips});

  ProjectState copyWith({String? title, List<MediaClip>? clips}) {
    return ProjectState(title: title ?? this.title, clips: clips ?? this.clips);
  }
}

/// Notifier for managing the project state.
class ProjectNotifier extends StateNotifier<ProjectState> {
  ProjectNotifier() : super(ProjectState(title: "Untitled Project", clips: []));

  void addClip(MediaClip clip) {
    state = state.copyWith(clips: [...state.clips, clip]);
  }

  void removeClip(String id) {
    state = state.copyWith(
      clips: state.clips.where((clip) => clip.id != id).toList(),
    );
  }

  void clearClips() {
    state = state.copyWith(clips: []);
  }

  void setTitle(String title) {
    state = state.copyWith(title: title);
  }
}

/// Riverpod provider for the project state.
final projectProvider = StateNotifierProvider<ProjectNotifier, ProjectState>((
  ref,
) {
  return ProjectNotifier();
});
