import os

def count_files(directory):
    file_counts = {
        'jxl': 0,
        'png': 0,
        'jpg': 0,
        'jpeg': 0,
        'webp': 0
    }

    for root, dirs, files in os.walk(directory):
        for file in files:
            if file.endswith('.jxl'):
                file_counts['jxl'] += 1
            elif file.endswith('.png'):
                file_counts['png'] += 1
            elif file.endswith('.jpg'):
                file_counts['jpg'] += 1
            elif file.endswith('.jpeg'):
                file_counts['jpeg'] += 1
            elif file.endswith('.webp'):
                file_counts['webp'] += 1

    return file_counts

def main():
    current_directory = os.getcwd()
    directories = [d for d in os.listdir(current_directory) if os.path.isdir(os.path.join(current_directory, d))]

    for directory in directories:
        dir_path = os.path.join(current_directory, directory)
        counts = count_files(dir_path)
        total_files = sum(counts.values())
        
        if total_files > 0:
            print(f"Directory: {directory}")
            if counts['jxl'] > 0:
                print(f"*.jxl files: {counts['jxl']}")
            if counts['png'] > 0:
                print(f"*.png files: {counts['png']}")
            if counts['jpg'] > 0:
                print(f"*.jpg files: {counts['jpg']}")
            if counts['jpeg'] > 0:
                print(f"*.jpeg files: {counts['jpeg']}")
            if counts['webp'] > 0:
                print(f"*.webp files: {counts['webp']}")
            print(f"Total files: {total_files}")
            print()

if __name__ == "__main__":
    main()

