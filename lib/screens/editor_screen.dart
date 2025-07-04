import 'package:flutter/material.dart';
import '../widgets/video_preview.dart';
import '../widgets/timeline.dart';
import '../widgets/controls.dart';
import '../widgets/side_panel.dart';

class EditorScreen extends StatelessWidget {
  const EditorScreen({super.key});

  @override
  Widget build(BuildContext context) {
    // Main layout: Side panel | Main area (Video preview + Timeline + Controls)
    return Scaffold(
      backgroundColor: const Color(0xFFF6F7FB),
      appBar: AppBar(
        title: const Text(
          'Video Editor',
          style: TextStyle(fontWeight: FontWeight.w600, letterSpacing: 0.5),
        ),
        elevation: 0.5,
        backgroundColor: Colors.white,
        foregroundColor: Colors.black87,
        centerTitle: false,
      ),
      body: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Side Panel
          Container(
            decoration: const BoxDecoration(
              color: Colors.white,
              border: Border(right: BorderSide(color: Color(0xFFE0E0E0))),
            ),
            width: 220,
            child: Column(
              children: [
                const SizedBox(height: 24),
                Icon(
                  Icons.video_library_outlined,
                  size: 40,
                  color: Colors.deepPurple.shade200,
                ),
                const SizedBox(height: 12),
                const Text(
                  'Project',
                  style: TextStyle(
                    fontWeight: FontWeight.w600,
                    fontSize: 18,
                    color: Colors.black87,
                  ),
                ),
                const SizedBox(height: 8),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16.0),
                  child: OutlinedButton.icon(
                    onPressed: () {},
                    icon: const Icon(Icons.upload_file_rounded, size: 18),
                    label: const Text('Import Media'),
                    style: OutlinedButton.styleFrom(
                      minimumSize: const Size.fromHeight(36),
                      foregroundColor: Colors.deepPurple,
                      side: const BorderSide(color: Color(0xFFD1C4E9)),
                      textStyle: const TextStyle(fontWeight: FontWeight.w500),
                    ),
                  ),
                ),
                const Spacer(),
                const Padding(
                  padding: EdgeInsets.only(bottom: 16.0),
                  child: Text(
                    'No clips yet',
                    style: TextStyle(color: Colors.grey, fontSize: 13),
                  ),
                ),
              ],
            ),
          ),
          // Main Editing Area
          Expanded(
            child: Padding(
              padding: const EdgeInsets.symmetric(
                vertical: 24.0,
                horizontal: 32.0,
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  // Video Preview Card
                  Card(
                    elevation: 2,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(14),
                    ),
                    margin: EdgeInsets.zero,
                    child: Padding(
                      padding: const EdgeInsets.all(24.0),
                      child: Column(
                        children: [
                          const VideoPreview(),
                          const SizedBox(height: 16),
                          Row(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              Icon(
                                Icons.info_outline,
                                color: Colors.deepPurple.shade100,
                                size: 20,
                              ),
                              const SizedBox(width: 8),
                              Text(
                                'No media loaded. Import video or audio to get started.',
                                style: TextStyle(
                                  color: Colors.grey[500],
                                  fontSize: 15,
                                  fontStyle: FontStyle.italic,
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 28),
                  // Timeline Card
                  Card(
                    elevation: 1,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                    margin: EdgeInsets.zero,
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        vertical: 10.0,
                        horizontal: 8.0,
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          const Padding(
                            padding: EdgeInsets.only(left: 8.0, bottom: 2),
                            child: Text(
                              'Timeline',
                              style: TextStyle(
                                fontWeight: FontWeight.w500,
                                color: Colors.black54,
                                fontSize: 15,
                              ),
                            ),
                          ),
                          const Timeline(),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 18),
                  // Controls Card
                  Card(
                    elevation: 1,
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                    margin: EdgeInsets.zero,
                    child: const Padding(
                      padding: EdgeInsets.symmetric(
                        vertical: 6.0,
                        horizontal: 8.0,
                      ),
                      child: Controls(),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
