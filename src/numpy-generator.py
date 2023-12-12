import numpy as np
import os

def create_and_save_arrays():
    # Define the relative path to the data folder from the src folder
    data_folder_path = os.path.join(".", "data")

    # Create two sample arrays
    array_1 = np.array([[1, 2, 3], [4, 5, 6]])
    array_2 = np.array([1.0, 3.5, 5.5, 7.0])
    array_3 = np.array([1, 2, 3, 3, 3, 2, 8])
    array_4 = np.array([[1, 2, 3], [4, 5, 6],[7, 8 ,9]])
    array_5 = np.array([[[1, 2, 3], [4, 5, 6], [7, 8, 9]],
                     [[10, 11, 12], [13, 14, 15], [16, 17, 18]],
                     [[19, 20, 21], [22, 23, 24], [25, 26, 27]]]) # 3D
    array_6 = np.array([[[[ 1,  2,  3],
                        [ 4,  5,  6],
                        [ 7,  8,  9]],

                        [[10, 11, 12],
                        [13, 14, 15],
                        [16, 17, 18]],

                        [[19, 20, 21],
                        [22, 23, 24],
                        [25, 26, 27]]],

                    [[[28, 29, 30],
                        [31, 32, 33],
                        [34, 35, 36]],

                        [[37, 38, 39],
                        [40, 41, 42],
                        [43, 44, 45]],

                        [[46, 47, 48],
                        [49, 50, 51],
                        [52, 53, 54]]]]) # 4D

    # Check if the data directory exists, create if it doesn't
    if not os.path.exists(data_folder_path):
        os.makedirs(data_folder_path)

    # Save the arrays to .npy files in the data directory
    np.save(os.path.join(data_folder_path, 'array_1.npy'), array_1)
    np.save(os.path.join(data_folder_path, 'array_2.npy'), array_2)
    np.save(os.path.join(data_folder_path, 'array_3.npy'), array_3)
    np.save(os.path.join(data_folder_path, 'array_4.npy'), array_4)
    np.save(os.path.join(data_folder_path, 'array_5.npy'), array_5)
    np.save(os.path.join(data_folder_path, 'array_6.npy'), array_6)

    print("Arrays saved in the data folder")

create_and_save_arrays()