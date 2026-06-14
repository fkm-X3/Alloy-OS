import QtQuick
import QtQuick.Controls

Rectangle {
    id: appLauncherRoot
    color: Qt.rgba(0, 0, 0, 0.85)

    signal appLaunched(string name)

    MouseArea {
        anchors.fill: parent
        onClicked: parent.visible = false
    }

    Grid {
        anchors.centerIn: parent
        columns: 4
        spacing: 16

        Repeater {
            model: [
                { name: "Terminal", icon: "\u2B21" },
                { name: "Files",    icon: "\u2756" },
                { name: "Settings", icon: "\u2699" },
                { name: "Browser",  icon: "\u2B22" }
            ]

            delegate: Item {
                width: 100
                height: 100

                Column {
                    anchors.centerIn: parent
                    spacing: 8

                    Rectangle {
                        anchors.horizontalCenter: parent.horizontalCenter
                        width: 64
                        height: 64
                        radius: 12
                        color: "#2a2a4e"

                        Text {
                            anchors.centerIn: parent
                            text: modelData.icon
                            font.pixelSize: 28
                            color: "#ffffff"
                        }

                        MouseArea {
                            anchors.fill: parent
                            onClicked: {
                                appLauncherRoot.visible = false
                                appLauncherRoot.appLaunched(modelData.name)
                            }
                        }
                    }

                    Text {
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: modelData.name
                        color: "#cccccc"
                        font.pixelSize: 12
                    }
                }
            }
        }
    }

    Shortcut {
        sequence: "Escape"
        onActivated: parent.visible = false
    }
}
