import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    color: "#1a1a2e"

    RowLayout {
        anchors { fill: parent; margins: 4 }
        spacing: 4

        Button {
            id: launcherBtn
            Layout.preferredWidth: 40
            Layout.fillHeight: true
            text: "\u2302"
            font.pixelSize: 18
            flat: true

            background: Rectangle {
                color: launcherBtn.hovered ? "#2a2a4e" : "transparent"
                radius: 6
            }

            contentItem: Text {
                text: launcherBtn.text
                font: launcherBtn.font
                color: "#ffffff"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }

            onClicked: launcher.visible = !launcher.visible
        }

        Item {
            Layout.fillWidth: true
        }

        Label {
            id: clockLabel
            Layout.rightMargin: 12
            color: "#ffffff"
            font.pixelSize: 14
            verticalAlignment: Text.AlignVCenter
            text: new Date().toLocaleTimeString(Qt.locale(), "hh:mm")
        }

        Button {
            id: quitBtn
            Layout.preferredWidth: 40
            Layout.fillHeight: true
            text: "\u2715"
            font.pixelSize: 14
            flat: true

            background: Rectangle {
                color: quitBtn.hovered ? "#4e1a1a" : "transparent"
                radius: 6
            }

            contentItem: Text {
                text: quitBtn.text
                font: quitBtn.font
                color: "#ff6666"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }

            onClicked: Qt.quit()
        }
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: {
            clockLabel.text = new Date().toLocaleTimeString(Qt.locale(), "hh:mm")
        }
    }
}
