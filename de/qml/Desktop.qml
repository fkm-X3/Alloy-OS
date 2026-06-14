import QtQuick
import QtQuick.Controls

Rectangle {
    color: "#0f0f1a"

    Column {
        anchors {
            right: parent.right
            bottom: parent.bottom
            margins: 24
        }
        spacing: 4

        Label {
            id: timeLabel
            color: "#ffffff"
            font.pixelSize: 64
            font.weight: Font.Light
            text: new Date().toLocaleTimeString(Qt.locale(), "hh:mm")
        }

        Label {
            id: dateLabel
            color: "#aaaaaa"
            font.pixelSize: 16
            text: new Date().toLocaleDateString(Qt.locale(), "dddd, MMMM d")
        }
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: {
            var now = new Date()
            timeLabel.text = now.toLocaleTimeString(Qt.locale(), "hh:mm")
            dateLabel.text = now.toLocaleDateString(Qt.locale(), "dddd, MMMM d")
        }
    }
}
