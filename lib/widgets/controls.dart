import 'package:flutter/material.dart';

class Controls extends StatelessWidget {
  const Controls({super.key});

  @override
  Widget build(BuildContext context) {
    final Color iconColor = Colors.grey.shade800;
    final Color iconActive = Colors.deepPurpleAccent.shade100;
    final Color sliderBg = Colors.grey.shade200;

    return Container(
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 8),
      child: Row(
        children: [
          _FlatIconButton(
            icon: Icons.skip_previous_rounded,
            tooltip: 'Previous',
            color: iconColor,
            onTap: () {},
          ),
          _FlatIconButton(
            icon: Icons.play_arrow_rounded,
            tooltip: 'Play',
            color: iconActive,
            onTap: () {},
          ),
          _FlatIconButton(
            icon: Icons.pause_rounded,
            tooltip: 'Pause',
            color: iconColor,
            onTap: () {},
          ),
          _FlatIconButton(
            icon: Icons.stop_rounded,
            tooltip: 'Stop',
            color: iconColor,
            onTap: () {},
          ),
          _FlatIconButton(
            icon: Icons.skip_next_rounded,
            tooltip: 'Next',
            color: iconColor,
            onTap: () {},
          ),
          const SizedBox(width: 12),
          Flexible(
            flex: 1,
            child: Container(
              height: 28,
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: sliderBg,
                borderRadius: BorderRadius.circular(6),
              ),
              child: SliderTheme(
                data: SliderTheme.of(context).copyWith(
                  trackHeight: 3,
                  thumbShape: const RoundSliderThumbShape(
                    enabledThumbRadius: 7,
                  ),
                  overlayShape: SliderComponentShape.noOverlay,
                  activeTrackColor: Colors.deepPurpleAccent.shade100,
                  inactiveTrackColor: Colors.grey.shade400,
                  thumbColor: Colors.deepPurpleAccent.shade100,
                ),
                child: Slider(
                  value: 0,
                  min: 0,
                  max: 100,
                  onChanged: (double value) {},
                ),
              ),
            ),
          ),
          const SizedBox(width: 8),
          ConstrainedBox(
            constraints: const BoxConstraints(minWidth: 40, maxWidth: 48),
            child: Text(
              "00:00",
              overflow: TextOverflow.fade,
              softWrap: false,
              style: TextStyle(
                color: Colors.grey.shade600,
                fontSize: 13,
                fontFeatures: const [FontFeature.tabularFigures()],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _FlatIconButton extends StatefulWidget {
  final IconData icon;
  final String tooltip;
  final Color color;
  final VoidCallback onTap;

  const _FlatIconButton({
    required this.icon,
    required this.tooltip,
    required this.color,
    required this.onTap,
    super.key,
  });

  @override
  State<_FlatIconButton> createState() => _FlatIconButtonState();
}

class _FlatIconButtonState extends State<_FlatIconButton> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final Color bg = _hovering
        ? widget.color.withOpacity(0.08)
        : Colors.transparent;
    final Color iconColor = _hovering
        ? widget.color
        : widget.color.withOpacity(0.85);

    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: Tooltip(
        message: widget.tooltip,
        child: InkWell(
          borderRadius: BorderRadius.circular(6),
          onTap: widget.onTap,
          child: Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              color: bg,
              borderRadius: BorderRadius.circular(6),
            ),
            child: Icon(widget.icon, color: iconColor, size: 22),
          ),
        ),
      ),
    );
  }
}
