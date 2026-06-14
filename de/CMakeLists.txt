cmake_minimum_required(VERSION 3.16)
project(AlloyDE VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Quick Qml)

qt_add_executable(AlloyDE
    src/main.cpp
    src/TerminalProcess.cpp
)

qt_add_resources(AlloyDE app_resources
    PREFIX "/"
    FILES
        qml/main.qml
        qml/Desktop.qml
        qml/Panel.qml
        qml/AppLauncher.qml
        qml/Terminal.qml
)

target_link_libraries(AlloyDE PRIVATE Qt6::Quick Qt6::Qml)

get_target_property(_qt6_qmake Qt6::qmake IMPORTED_LOCATION)
get_filename_component(QT6_BIN_DIR "${_qt6_qmake}" DIRECTORY)
find_program(WINDEPLOYQT windeployqt HINTS "${QT6_BIN_DIR}" REQUIRED)

add_custom_command(TARGET AlloyDE POST_BUILD
    COMMAND ${WINDEPLOYQT}
        --qmldir "${CMAKE_SOURCE_DIR}/qml"
        "$<TARGET_FILE:AlloyDE>"
    WORKING_DIRECTORY $<TARGET_FILE_DIR:AlloyDE>
    COMMENT "Deploying Qt libraries and QML imports..."
)
