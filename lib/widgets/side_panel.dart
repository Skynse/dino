import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/project_provider.dart';

class SidePanel extends ConsumerWidget {
  const SidePanel({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final project = ref.watch(projectProvider);

    void addMockClip(MediaType type) {
      final id = DateTime.now().millisecondsSinceEpoch.toString();
      ref
          .read(projectProvider.notifier)
          .addClip(
            MediaClip(
              id: id,
              filePath: type == MediaType.video
                  ? 'video_$id.mp4'
                  : 'audio_$id.mp3',
              start: Duration.zero,
              end: const Duration(seconds: 10),
              type: type,
            ),
          );
    }

    return Container(
      width: 220,
      decoration: const BoxDecoration(
        color: Color(0xFFF8F9FB),
        border: Border(right: BorderSide(color: Color(0xFFE5E7EB), width: 1)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const SizedBox(height: 24),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20.0),
            child: Row(
              children: [
                Icon(
                  Icons.video_library_outlined,
                  color: Color(0xFF6366F1),
                  size: 22,
                ),
                const SizedBox(width: 8),
                const Text(
                  'Project',
                  style: TextStyle(
                    fontWeight: FontWeight.w600,
                    fontSize: 16,
                    color: Color(0xFF18181B),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 18),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20.0),
            child: Row(
              children: [
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: () => addMockClip(MediaType.video),
                    icon: const Icon(
                      Icons.video_file,
                      size: 18,
                      color: Color(0xFF6366F1),
                    ),
                    label: const Text('Add Video'),
                    style: OutlinedButton.styleFrom(
                      side: const BorderSide(
                        color: Color(0xFF6366F1),
                        width: 1,
                      ),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(6),
                      ),
                      padding: const EdgeInsets.symmetric(
                        vertical: 12,
                        horizontal: 8,
                      ),
                      backgroundColor: Colors.transparent,
                      elevation: 0,
                      textStyle: const TextStyle(fontSize: 14),
                      foregroundColor: Color(0xFF6366F1),
                    ),
                  ),
                ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20.0, vertical: 8),
            child: Row(
              children: [
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: () => addMockClip(MediaType.audio),
                    icon: const Icon(
                      Icons.audiotrack,
                      size: 18,
                      color: Color(0xFF6366F1),
                    ),
                    label: const Text('Add Audio'),
                    style: OutlinedButton.styleFrom(
                      side: const BorderSide(
                        color: Color(0xFF6366F1),
                        width: 1,
                      ),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(6),
                      ),
                      padding: const EdgeInsets.symmetric(
                        vertical: 12,
                        horizontal: 8,
                      ),
                      backgroundColor: Colors.transparent,
                      elevation: 0,
                      textStyle: const TextStyle(fontSize: 14),
                      foregroundColor: Color(0xFF6366F1),
                    ),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 32),
          const Divider(
            color: Color(0xFFE5E7EB),
            thickness: 1,
            height: 1,
            indent: 0,
            endIndent: 0,
          ),
          const SizedBox(height: 16),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 20.0),
            child: Text(
              project.clips.isEmpty
                  ? 'No clips yet'
                  : '${project.clips.length} clip(s) added',
              style: TextStyle(
                color: Colors.grey[500],
                fontSize: 13,
                fontStyle: FontStyle.italic,
                fontWeight: FontWeight.w400,
              ),
            ),
          ),
          const Spacer(),
        ],
      ),
    );
  }
}
