import QtQuick
import QtQuick.Controls

Window {
    id: root
    visible: true
    visibility: Window.FullScreen
    color: "#000000"

    Desktop {
        id: desktop
        anchors.fill: parent
    }

    AppLauncher {
        id: launcher
        anchors.fill: parent
        visible: false

        onAppLaunched: function(name) {
            if (name === "Terminal")
                terminal.visible = true
        }
    }

    Terminal {
        id: terminal
        anchors {
            top: parent.top
            left: parent.left
            right: parent.right
            bottom: parent.bottom
            margins: 16
        }
        visible: false

        onClosed: {
            launcher.visible = false
        }
    }

    Shortcut {
        sequence: "Escape"
        onActivated: launcher.visible = false
    }

    Shortcut {
        sequence: "Meta"
        onActivated: launcher.visible = !launcher.visible
    }
}
