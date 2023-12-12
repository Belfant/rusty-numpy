import numpy as np
import os

def create_and_save_arrays():
    # Define the relative path to the data folder from the src folder
    data_folder_path = os.path.join(".", "data")

    # Create two sample arrays
    array_1 = np.array([[1, 2, 3], [4, 5, 6]])
    array_2 = np.array([1.0, 3.5, 5.5, 7.0])
    array_3 = np.array([1, 2, 3, 3, 3, 2, 8])

    # Check if the data directory exists, create if it doesn't
    if not os.path.exists(data_folder_path):
        os.makedirs(data_folder_path)

    # Save the arrays to .npy files in the data directory
    np.save(os.path.join(data_folder_path, 'array_1.npy'), array_1)
    np.save(os.path.join(data_folder_path, 'array_2.npy'), array_2)
    np.save(os.path.join(data_folder_path, 'array_3.npy'), array_3)

    print("Arrays saved in the data folder")

create_and_save_arrays()