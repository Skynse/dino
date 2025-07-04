import 'package:flutter/material.dart';

class VideoPreview extends StatelessWidget {
  final String? videoPath;

  const VideoPreview({Key? key, this.videoPath}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    // Flat, modern, neutral look for empty video preview
    return AspectRatio(
      aspectRatio: 16 / 9,
      child: Container(
        decoration: BoxDecoration(
          color: const Color(0xFF18181B),
          border: Border.all(color: const Color(0xFF27272A), width: 1),
          borderRadius: BorderRadius.circular(6),
        ),
        child: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                Icons.videocam_outlined,
                color: Colors.white.withOpacity(0.18),
                size: 56,
              ),
              const SizedBox(height: 10),
              Text(
                'No video loaded',
                style: TextStyle(
                  color: Colors.white.withOpacity(0.32),
                  fontWeight: FontWeight.w500,
                  fontSize: 15,
                  letterSpacing: 0.1,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
