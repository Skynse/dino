import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/project_provider.dart';

class Timeline extends ConsumerWidget {
  const Timeline({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final clips = ref.watch(projectProvider).clips;

    return SizedBox(
      height: 56,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.only(left: 4, bottom: 2),
            child: Row(
              children: [
                Icon(Icons.timeline, size: 18, color: Colors.grey[400]),
                const SizedBox(width: 6),
                Text(
                  'Timeline',
                  style: TextStyle(
                    color: Colors.grey[700],
                    fontWeight: FontWeight.w500,
                    fontSize: 13.5,
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            child: Container(
              margin: const EdgeInsets.only(top: 2),
              decoration: BoxDecoration(
                color: Colors.white,
                border: Border.all(color: const Color(0xFFE5E7EB), width: 1),
                borderRadius: BorderRadius.circular(6),
              ),
              child: clips.isEmpty
                  ? Center(
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(
                            Icons.video_file_outlined,
                            color: Colors.grey[400],
                            size: 22,
                          ),
                          const SizedBox(width: 8),
                          Text(
                            'No clips in timeline',
                            style: TextStyle(
                              color: Colors.grey[400],
                              fontSize: 14,
                              fontStyle: FontStyle.italic,
                            ),
                          ),
                        ],
                      ),
                    )
                  : ListView.separated(
                      scrollDirection: Axis.horizontal,
                      padding: const EdgeInsets.symmetric(
                        horizontal: 12,
                        vertical: 8,
                      ),
                      itemCount: clips.length,
                      separatorBuilder: (_, __) => const SizedBox(width: 12),
                      itemBuilder: (context, i) {
                        final clip = clips[i];
                        return Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 16,
                            vertical: 8,
                          ),
                          decoration: BoxDecoration(
                            color: clip.type == MediaType.video
                                ? Colors.deepPurple.shade50
                                : Colors.teal.shade50,
                            borderRadius: BorderRadius.circular(6),
                            border: Border.all(
                              color: clip.type == MediaType.video
                                  ? Colors.deepPurple.shade100
                                  : Colors.teal.shade100,
                              width: 1.5,
                            ),
                          ),
                          child: Row(
                            children: [
                              Icon(
                                clip.type == MediaType.video
                                    ? Icons.video_file
                                    : Icons.audiotrack,
                                color: clip.type == MediaType.video
                                    ? Colors.deepPurple
                                    : Colors.teal,
                                size: 20,
                              ),
                              const SizedBox(width: 8),
                              Text(
                                clip.filePath,
                                style: TextStyle(
                                  color: Colors.grey[800],
                                  fontWeight: FontWeight.w500,
                                  fontSize: 13,
                                ),
                              ),
                            ],
                          ),
                        );
                      },
                    ),
            ),
          ),
        ],
      ),
    );
  }
}
