import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import AlloyDE.Terminal 1.0

Rectangle {
    id: root
    color: "#0d0d1a"
    radius: 8

    property bool ready: false

    signal closed()

    onVisibleChanged: {
        if (visible) {
            root.ready = true
            outputText.text = ""
            proc.start()
            inputField.forceActiveFocus()
        }
    }

    TerminalProcess {
        id: proc

        onOutputReceived: function(text) {
            outputText.text += text
            scrollTimer.restart()
        }
    }

    Timer {
        id: scrollTimer
        interval: 50
        onTriggered: {
            outputFlickable.contentY = outputFlickable.contentHeight - outputFlickable.height
        }
    }

    Rectangle {
        id: titleBar
        anchors { top: parent.top; left: parent.left; right: parent.right }
        height: 32
        color: "#1a1a2e"
        radius: 8

        Rectangle {
            anchors { top: parent.top; left: parent.left; right: parent.right }
            height: 8
            color: parent.color
        }

        Text {
            anchors { left: parent.left; leftMargin: 12; verticalCenter: parent.verticalCenter }
            text: "Terminal"
            color: "#888899"
            font.pixelSize: 12
        }

        Button {
            anchors { right: parent.right; rightMargin: 4; verticalCenter: parent.verticalCenter }
            width: 24; height: 24
            text: "\u2715"
            font.pixelSize: 12
            flat: true

            background: Rectangle {
                color: parent.hovered ? "#4e1a1a" : "transparent"
                radius: 4
            }

            contentItem: Text {
                text: parent.text
                font: parent.font
                color: "#ff6666"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }

            onClicked: root.closeTerminal()
        }
    }

    Flickable {
        id: outputFlickable
        anchors {
            top: titleBar.bottom
            left: parent.left
            right: parent.right
            bottom: inputRow.top
            margins: 8
        }
        contentHeight: outputText.height + 10
        clip: true

        TextEdit {
            id: outputText
            width: outputFlickable.width
            color: "#00cc66"
            font.family: "Consolas"
            font.pixelSize: 14
            font.weight: Font.Normal
            readOnly: true
            selectByMouse: true
            wrapMode: TextEdit.Wrap
            textFormat: TextEdit.PlainText
            padding: 4
        }

        ScrollBar.vertical: ScrollBar {
            policy: ScrollBar.AsNeeded
            width: 8
            background: Rectangle { color: "#1a1a2e"; radius: 4 }
            contentItem: Rectangle {
                color: "#3a3a5e"
                radius: 4
            }
        }
    }

    Rectangle {
        id: inputRow
        anchors {
            left: parent.left
            right: parent.right
            bottom: parent.bottom
            margins: 8
        }
        height: 36
        color: "#0a0a15"
        radius: 6

        RowLayout {
            anchors { fill: parent; margins: 8 }
            spacing: 6

            Text {
                color: "#00cc66"
                font.family: "Consolas"
                font.pixelSize: 14
                text: "$ "
                verticalAlignment: Text.AlignVCenter
            }

            TextField {
                id: inputField
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: "#e0e0e0"
                font.family: "Consolas"
                font.pixelSize: 14
                background: null
                verticalAlignment: Text.AlignVCenter
                selectByMouse: true

                onAccepted: {
                    var cmd = inputField.text
                    if (cmd.trim() !== "") {
                        outputText.text += "$ " + cmd + "\n"
                        proc.write(cmd)
                    } else {
                        outputText.text += "$ \n"
                    }
                    inputField.text = ""
                }

                Keys.onEscapePressed: {
                    root.closeTerminal()
                }
            }
        }
    }

    Shortcut {
        sequence: "Escape"
        onActivated: {
            if (root.visible)
                root.closeTerminal()
        }
    }

    function closeTerminal() {
        root.visible = false
        proc.stop()
        root.closed()
    }

    Component.onCompleted: {
        if (root.visible) {
            proc.start()
            inputField.forceActiveFocus()
        }
    }

    Component.onDestruction: {
        proc.stop()
    }
}
