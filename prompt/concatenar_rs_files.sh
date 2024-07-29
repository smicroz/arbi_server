#!/bin/bash

# Directorio de salida para el script
SCRIPT_DIR="./"

# Directorio a leer
SRC_DIR="../src"

# Nombre del archivo de salida
OUTPUT_FILE="${SCRIPT_DIR}/arbit_server_rust_files_content.txt"

# Crear el directorio de salida si no existe
mkdir -p "$SCRIPT_DIR"

# Limpiar el archivo de salida si ya existe
> "$OUTPUT_FILE"

# Encontrar todos los archivos .rs y escribir su contenido en el archivo de salida
find "$SRC_DIR" -type f -name "*.rs" | while read -r file; do
    # Obtener la ruta relativa del archivo dentro de src
    relative_path=${file#"$SRC_DIR/"}
    
    # Escribir la ruta del archivo en el archivo de salida
    echo "Ruta: $relative_path" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    # Escribir el contenido del archivo
    cat "$file" >> "$OUTPUT_FILE"
    
    # Agregar un separador entre archivos
    echo -e "\n\n--------------------\n\n" >> "$OUTPUT_FILE"
done

echo "Proceso completado. El contenido se ha guardado en $OUTPUT_FILE"